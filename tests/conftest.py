import pytest

from pyforwarder import PyForwarder


@pytest.fixture(scope="session")
def forwarder():
    with PyForwarder():
        yield
