import os
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple

import yaml


@dataclass(frozen=True)
class Service:
    name: str
    type: str
    service_host: str
    service_port: int
    local_port: int


@dataclass(frozen=True)
class SSHConfig:
    ssh_port: int
    username: str
    key_filename: str
    client_interface: str


class Services:
    def __init__(self, path: Optional[str] = None):
        self.ssh_config, self.services = self.parse_services(path)

    def __iter__(self):
        return iter(self.services)

    def __len__(self):
        return len(self.services)

    def get_service(self, service_name: str):
        services = list(filter(lambda s: s.name == service_name, self.services))
        if len(services) == 0:
            raise ValueError("Service not found. Please provide a correct name")

        return services[0]

    def parse_services(
        self, user_config_path: Optional[str] = None
    ) -> Tuple[SSHConfig, List[Service]]:
        config = get_config(user_config_path)
        ssh_config, services_config = config["ssh_config"], config["services"]
        return SSHConfig(**ssh_config), [Service(**s) for s in services_config]


def _merge_config(d1: Dict[str, Any], d2: Dict[str, Any]) -> None:
    for k, v2 in d2.items():
        v1 = d1.get(k)
        if isinstance(v1, dict) and isinstance(v2, dict):
            _merge_config(v1, v2)
        elif isinstance(v1, list):
            d1[k].extend(v2)
        else:
            d1[k] = v2


def get_config(user_config_path: Optional[str] = None) -> Dict[str, Any]:
    # NOTE:
    user_config_path = os.environ.get("FORWARDER_CONFIG") or user_config_path

    user_config = {}
    if user_config_path:
        with open(user_config_path, "r") as file:
            user_config = yaml.safe_load(file)
        return user_config
    # TODO : DO this better, include yaml config in package or parse correctly
    # Parse default Yaml
    config_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "config.yml")
    with open(config_path, "r") as file:
        config = yaml.safe_load(file)
    _merge_config(config, user_config)
    return config
