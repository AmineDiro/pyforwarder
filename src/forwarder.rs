use std::fs;
use std::io;
use std::path::PathBuf;

use futures::future::join_all;
use makiko::{self, Client, ClientEvent, ClientReceiver};
use tokio::{self, net::ToSocketAddrs};

use crate::ssh_proxy::{SSHProxy, SSHProxyConfig};

async fn authenticate(
    client: &Client,
    username: String,
    priv_key_path: PathBuf,
    pub_key_algo: String,
) {
    // Read the private key
    let priv_key = fs::read(priv_key_path).expect("Private key can't be read from path");
    // Decode our private key from PEM.
    let priv_key = makiko::keys::decode_pem_privkey_nopass(&priv_key)
        .expect("Could not decode a private key from PEM")
        .privkey()
        .cloned()
        .expect("Private key is encrypted");

    let pub_key_algo = match pub_key_algo.as_str() {
        "RSA_256" => &makiko::pubkey::RSA_SHA2_256,
        _ => panic!("Can't connect using this public key algorithm"),
    };

    // Try to authenticate with the private key
    let auth_res = client
        .auth_pubkey(username.into(), priv_key, pub_key_algo)
        .await
        .expect("Error when trying to authenticate");
    // Deal with the possible outcomes of public key authentication.
    match auth_res {
        makiko::AuthPubkeyResult::Success => {
            log::info!("We have successfully authenticated using a private key");
        }
        makiko::AuthPubkeyResult::Failure(failure) => {
            panic!("The server rejected authentication: {:?}", failure);
        }
    }
}

pub struct Forwarder {
    pub client: Client,
    proxies: Vec<SSHProxy>,
}
impl Forwarder {
    pub async fn new(
        remote_addr: impl ToSocketAddrs,
        username: String,
        priv_key_path: PathBuf,
        pub_key_algo: String,
        proxies_config: Vec<SSHProxyConfig>,
    ) -> Self {
        let socket = tokio::net::TcpStream::connect(remote_addr)
            .await
            .expect("Could not open a TCP socket");
        let config = makiko::ClientConfig::default();
        let (client, client_rx, client_fut) =
            makiko::Client::open(socket, config).expect("Could not open client");
        // To handle the SSH connection, we need to asynchronously run the code that performs I/O on the underlying socket
        tokio::task::spawn(async move {
            client_fut.await.expect("error in makiko::client_fut");
        });
        // Handle event in a separate task
        let _client_events = tokio::task::spawn(client_event_loop(client_rx));

        // Try to authenticate using a password.
        authenticate(&client, username, priv_key_path, pub_key_algo).await;
        // Build proxies
        let f_proxies = proxies_config
            .into_iter()
            .map(|c| SSHProxy::new(client.clone(), c))
            .collect::<Vec<_>>();
        let proxies = join_all(f_proxies).await;

        Self { client, proxies }
    }

    pub async fn start(&self) -> io::Result<()> {
        // Loop over all the defined services and create the ssh proxy
        // Blocks here
        join_all(self.proxies.iter().map(|p| p.start()))
            .await
            .into_iter()
            .collect()
    }
}

async fn client_event_loop(mut client_rx: ClientReceiver) {
    while let Ok(Some(event)) = client_rx.recv().await {
        match event {
            ClientEvent::ServerPubkey(_pubkey, accept) => {
                log::info!("received pubkey from server. Accepting by default");
                accept.accept();
            }
            // Note:  you must receive these events,
            // otherwise the client will stall when the internal buffer of events fills up.
            _ => {}
        }
    }
}
