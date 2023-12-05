use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub n_workers: usize,
    pub ssh_config: SshConfig,
    pub services: Vec<Service>,
}
#[derive(Deserialize, Debug)]

pub struct SshConfig {
    pub ssh_server: String,
    pub ssh_port: u16,
    pub username: String,
    pub priv_key_path: String,
    pub pub_key_algo: String,
    pub client_interface: String,
}

#[derive(Deserialize, Debug)]
pub struct Service {
    pub name: String,
    pub service_host: String,
    pub service_port: String,
    pub local_port: u16,
}
