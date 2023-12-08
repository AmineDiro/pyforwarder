import logging

from aiohttp import web

routes = web.RouteTableDef()


@routes.get("/")
async def hello(request):
    return web.Response()


app = web.Application()
logging.basicConfig(level=logging.DEBUG)
app.add_routes(routes)
web.run_app(app, port=8181)
