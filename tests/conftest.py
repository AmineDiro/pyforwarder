from pathlib import Path

import pytest

from pyforwarder import PyForwarder


@pytest.fixture(scope="session")
def forwarder():
    path = Path("tests/config.yml")
    with PyForwarder(path):
        yield
