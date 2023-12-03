use std::io;

use futures::future::join_all;
use makiko::{self, Client, ClientEvent, ClientReceiver};
use tokio::{self, net::ToSocketAddrs};

use crate::ssh_proxy::{SSHProxy, SSHProxyConfig};

async fn authenticate(client: &Client, username: String, password: String) {
    // Try to authenticate using a password.
    let auth_res = client
        .auth_password(username, password)
        .await
        .expect("error when trying to authenticate");

    match auth_res {
        makiko::AuthPasswordResult::Success => {
            log::info!("successfully authenticated using a password");
        }
        makiko::AuthPasswordResult::ChangePassword(prompt) => {
            panic!("The server asks us to change password: {:?}", prompt);
        }
        makiko::AuthPasswordResult::Failure(failure) => {
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
        password: String,
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
        authenticate(&client, username, password).await;
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
