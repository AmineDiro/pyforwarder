import time

import pytest


@pytest.fixture(scope="session")
def simple_http_openssh(docker_services):
    """Ensure that SSH service is up and responsive."""
    # `port_for` takes a container port and returns the corresponding host port
    port = docker_services.port_for("simple-http", 22)
    url = f"http://localhost:{port}"
    # TODO: find a better way to check for container to spawn
    time.sleep(2)
    return url
