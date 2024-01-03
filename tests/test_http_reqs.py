from pathlib import Path

import aiohttp
import pytest
from tqdm.asyncio import tqdm_asyncio

from pyforwarder import PyForwarder


async def send_req(session: aiohttp.ClientSession, url: str):
    async with session.get(url) as resp:
        resp.raise_for_status()


@pytest.mark.parametrize("n", [10])
@pytest.mark.asyncio
async def test_simple_http(simple_http_openssh, n):
    service_url = "http://127.0.0.1:8181"

    path = Path("./tests/config.yml")
    with PyForwarder(path):
        async with aiohttp.ClientSession() as session:
            result = await tqdm_asyncio.gather(
                *[send_req(session, service_url) for _ in range(n)]
            )
        assert len(result) == n
