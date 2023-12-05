use forwarder::Forwarder;
use pyo3::{exceptions::PyValueError, prelude::*};
use std::{fs, path::PathBuf};
use tokio::{
    runtime::{self},
    sync::oneshot,
    task::JoinHandle,
};

use std::{net::SocketAddr, str::FromStr, time::Duration};
pub mod config;
pub mod forwarder;
pub mod ssh_proxy;

use config::Config;
use tokio::{self, sync::Semaphore};

use ssh_proxy::SSHProxyConfig;
static PERMITS: Semaphore = Semaphore::const_new(10_000);

#[pyclass]
struct PyForwarder {
    rt: Option<runtime::Runtime>,
    _handle: Option<JoinHandle<()>>,
}
/// Pyforwarder reads a config_path file and connects to
///
#[pymethods]
impl PyForwarder {
    #[new]
    #[pyo3(signature = (config_path))]
    fn new(config_path: PathBuf) -> PyResult<Self> {
        let config_data = fs::read_to_string(config_path)
            .map_err(|e| PyValueError::new_err(format!("Can't read config file. err: {:?}", e)))?;
        let config: Config = serde_yaml::from_str(&config_data)
            .map_err(|_| PyValueError::new_err("can't deserialize config"))?;
        let rt = runtime::Builder::new_multi_thread()
            .worker_threads(config.n_workers)
            .thread_name("forwarder-thread-tokio")
            .enable_io()
            .build()
            .unwrap();

        Ok(Self {
            rt: Some(rt),
            _handle: None,
        })
    }

    pub fn __enter__(&mut self, py: Python<'_>) {
        py.allow_threads(|| {
            let (tx, rx) = oneshot::channel::<()>();
            let handle = self.rt.as_mut().unwrap().spawn(async move {
                // Define the list of services we want to forward
                let proxies = vec![SSHProxyConfig {
                    name: Some("simple_http".into()),
                    local_addr: SocketAddr::from_str("127.0.0.1:8181").unwrap(),
                    remote_addr: SocketAddr::from_str("127.0.0.1:8181").unwrap(),
                }];

                // Connect to server
                let forwarder = Forwarder::new(
                    ("localhost", 2222),
                    "amine".into(),
                    "/home/amine/Documents/coding/pyforwarder/tests/keys/amine_rsa".into(),
                    "RSA_256".into(),
                    proxies,
                )
                .await;
                // Note: once we have created the forwarder => we have binded start the ssh_proxies
                // we can signal to the task that we can start receiving requests
                tx.send(()).unwrap();

                forwarder.start().await.unwrap()
            });
            match rx.blocking_recv() {
                Ok(_) => self._handle = Some(handle),
                Err(_) => panic!("can't start forwarder"),
            }
        })
    }

    pub fn __exit__(&mut self, _exc_type: PyObject, _exc_value: PyObject, _traceback: PyObject) {
        if let Some(handle) = &self._handle {
            handle.abort()
        }

        if let Some(rt) = self.rt.take() {
            rt.shutdown_timeout(Duration::from_secs(1));
        };

        log::info!("Shutting down port forwarding");
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn forwardrs(_py: Python, m: &PyModule) -> PyResult<()> {
    // Setup logging
    pyo3_log::init();

    m.add_class::<PyForwarder>()?;

    Ok(())
}
