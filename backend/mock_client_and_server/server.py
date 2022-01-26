#!/usr/bin/env python3
from http.server import HTTPServer, BaseHTTPRequestHandler
import os
import time

base_path = os.path.dirname(__file__)

class StaticServer(BaseHTTPRequestHandler):

    def execute_request(self):
        filename = 'mockresponse'

        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        with open(os.path.join(base_path, filename), 'rb') as fh:
            self.wfile.write(fh.read())

    def do_GET(self):
        time.sleep(0.05)
        self.execute_request()

def run(server_class=HTTPServer, handler_class=StaticServer, port=8000):
    server_address = ('', port)
    httpd = server_class(server_address, handler_class)
    print('Starting Server on port {}'.format(port))
    httpd.serve_forever()

run()