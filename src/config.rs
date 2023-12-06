use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub n_workers: usize,
    pub ssh_config: SSHConfig,
    pub proxy_config: Vec<SSHProxyConfig>,
}
#[derive(Deserialize, Debug)]

pub struct SSHConfig {
    pub ssh_server: String,
    pub ssh_port: u16,
    pub username: String,
    pub priv_key_path: String,
    pub pub_key_algo: String,
    pub client_interface: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SSHProxyConfig {
    pub name: String,
    pub service_ip: String,
    pub service_port: u16,
    pub local_ip: String,
    pub local_port: u16,
}
