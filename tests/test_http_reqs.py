import aiohttp
import pytest
from tqdm.asyncio import tqdm_asyncio

"""TODO:
This test should depend on the whole setup :
1. Build and start docker container:
    docker build -t openssh-server:v1 .
    docker run --name test --rm -p 2222:22 openssh-server:v1  
2. Start forwarder
    config= XXX
    with PyForwarder(config):
        yield
3. Run tests

"""


async def send_req(session: aiohttp.ClientSession, url: str):
    async with session.get(url) as resp:
        resp.raise_for_status()


@pytest.mark.parametrize("n", [10, 100, 1000])
@pytest.mark.asyncio
async def test_simple_http(n, forwarder):
    url = "http://127.0.0.1:8181"
    async with aiohttp.ClientSession() as session:
        result = await tqdm_asyncio.gather(*[send_req(session, url) for _ in range(n)])
    assert len(result) == n
