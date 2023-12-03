import logging

from pyforwarder import start_forwarder

FORMAT = "%(levelname)s %(name)s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)

start_forwarder()
