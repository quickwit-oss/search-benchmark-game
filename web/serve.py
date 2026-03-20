import gzip
import os
import sys
from functools import partial
from http.server import HTTPServer, SimpleHTTPRequestHandler


class GzipHTTPRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        if self.path.endswith(".json"):
            self.send_header("Content-Encoding", "gzip")
        super().end_headers()

    def do_GET(self):
        if self.path.endswith(".json"):
            path = self.translate_path(self.path)
            try:
                with open(path, "rb") as f:
                    content = f.read()
            except FileNotFoundError:
                self.send_error(404, "File not found")
                return
            compressed = gzip.compress(content)
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(compressed)))
            self.end_headers()
            self.wfile.write(compressed)
        else:
            super().do_GET()


port = int(sys.argv[1]) if len(sys.argv) > 1 else 8000
handler = partial(GzipHTTPRequestHandler, directory=os.getcwd())
httpd = HTTPServer(("", port), handler)
print(f"Serving on port {port} with gzip compression for JSON files")
httpd.serve_forever()
