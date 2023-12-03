use forwarder::Forwarder;
use pyo3::prelude::*;
use tokio::{runtime::Runtime, sync::oneshot, task::JoinHandle};

use std::{net::SocketAddr, str::FromStr};
pub mod forwarder;
pub mod ssh_proxy;

use tokio::{self, sync::Semaphore};

use ssh_proxy::SSHProxyConfig;
static PERMITS: Semaphore = Semaphore::const_new(10_000);

#[pyclass]
struct PyForwarder {
    _forwarding_handle: Option<JoinHandle<()>>,
}

#[pymethods]
impl PyForwarder {
    #[new]
    fn new() -> Self {
        Self {
            _forwarding_handle: None,
        }
    }

    pub fn __enter__(&mut self) {
        let runtime = Runtime::new().unwrap();

        let (tx, rx) = oneshot::channel::<()>();
        let handle = runtime.spawn(async move {
            // Define the list of services we want to forward
            let proxies = vec![SSHProxyConfig {
                name: Some("simple_http".into()),
                local_addr: SocketAddr::from_str("127.0.0.1:8181").unwrap(),
                remote_addr: SocketAddr::from_str("127.0.0.1:8181").unwrap(),
            }];

            // Connect to server
            let forwarder = Forwarder::new(
                ("localhost", 2222),
                "alice".into(),
                "alicealice".into(),
                proxies,
            )
            .await;
            forwarder.start().await.unwrap()
        });
        match rx.blocking_recv() {
            Ok(_) => self._forwarding_handle = Some(handle),
            Err(_) => println!("Can't start forwarder"),
        }
    }
    pub fn __exit__(&mut self, _exc_type: PyObject, _exc_value: PyObject, _traceback: PyObject) {
        if let Some(handle) = &self._forwarding_handle {
            handle.abort()
        }
        println!("exited");
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyforwarder(_py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<PyForwarder>()?;

    Ok(())
}
