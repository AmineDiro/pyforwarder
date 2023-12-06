# Pyforwarder

SSH port forwarding using Rust async runtime + python wrapping

## Usage :

1. Define a `config.yaml` config file:

```yaml
n_workers: 8

ssh_config:
  ssh_server: "localhost"
  ssh_port: 22
  username:
  priv_key_path:
  pub_key_algo:
  client_interface:

# Services you want to forward
services:
  - name: "http"
    service_host: "localhost"
    service_port: 8181
    local_port: 8181
```

2. Use in context :

```python
  with PyForwarder("./config.yml"):
      ....
```

# TODO:

- [ ] Parse IP from interface
- [ ] Change proxy to SSHconfig
