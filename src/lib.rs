use forwarder::Forwarder;
use pyo3::{exceptions::PyValueError, prelude::*};
use std::{fs, path::PathBuf};
use tokio::{
    runtime::{self},
    sync::oneshot,
    task::JoinHandle,
};

use std::time::Duration;
pub mod config;
pub mod forwarder;
pub mod ssh_proxy;

use config::{Config, SSHConfig, SSHProxyConfig};
use tokio::{self, sync::Semaphore};

static PERMITS: Semaphore = Semaphore::const_new(10_000);

#[pyclass]
struct PyForwarder {
    ssh_config: SSHConfig,
    proxy_config: Vec<SSHProxyConfig>,
    rt: Option<runtime::Runtime>,
    _handle: Option<JoinHandle<()>>,
}
/// Pyforwarder reads a config_path file and connects to
///
#[pymethods]
impl PyForwarder {
    #[new]
    #[pyo3(signature = (config_path))]
    fn new(config_path: PathBuf, py: Python<'_>) -> PyResult<Self> {
        py.allow_threads(|| {
            let config_data = fs::read_to_string(config_path).map_err(|e| {
                PyValueError::new_err(format!("Can't read config file. err: {:?}", e))
            })?;
            let Config {
                n_workers,
                ssh_config,
                proxy_config,
            }: Config = serde_yaml::from_str(&config_data)
                .map_err(|_| PyValueError::new_err("can't deserialize config"))?;
            let rt = runtime::Builder::new_multi_thread()
                .worker_threads(n_workers)
                .thread_name("forwarder-thread-tokio")
                .enable_io()
                .build()
                .unwrap();

            Ok(Self {
                ssh_config,
                proxy_config,
                rt: Some(rt),
                _handle: None,
            })
        })
    }

    pub fn __enter__(&mut self, py: Python<'_>) {
        py.allow_threads(|| {
            // Onshot channel to signal we have spawned the proxies
            let (tx, rx) = oneshot::channel::<()>();
            let mut forwarder = Forwarder::new(
                (self.ssh_config.ssh_server.clone(), self.ssh_config.ssh_port),
                self.ssh_config.username.clone(),
                self.ssh_config.priv_key_path.clone().into(),
                self.ssh_config.pub_key_algo.clone(),
                self.proxy_config.clone(),
            );

            let handle = self.rt.as_mut().unwrap().spawn(async move {
                // Note: once we have created the forwarder => we have binded start the ssh_proxies
                // we can signal to the task that we can start receiving requests
                forwarder.start(tx).await.unwrap()
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
