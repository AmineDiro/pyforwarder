import logging
import time

import requests

from pyforwarder import PyForwarder

FORMAT = "%(levelname)s %(name)s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)


with PyForwarder():
    resp = requests.get("http://127.0.0.1:8181")
    logging.info(resp)
