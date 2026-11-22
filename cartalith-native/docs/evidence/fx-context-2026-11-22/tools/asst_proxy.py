"""Recording loopback proxy: 127.0.0.1:<listen> -> 127.0.0.1:<target> (LM Studio). Logs request bodies and
streamed responses to a JSONL file. Never logs headers (no key exists for LM Studio anyway).
usage: python asst_proxy.py <listen_port> <target_port> <out.jsonl>"""
import http.client, json, sys, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

LISTEN, TARGET, OUT = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]


class H(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.0"

    def log_message(self, *a):
        pass

    def _go(self):
        n = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(n) if n else b""
        t0 = time.time()
        c = http.client.HTTPConnection("127.0.0.1", TARGET, timeout=900)
        hdr = {k: v for k, v in self.headers.items() if k.lower() not in ("host", "connection", "content-length")}
        c.request(self.command, self.path, body=body, headers=hdr)
        r = c.getresponse()
        self.send_response(r.status)
        for k, v in r.getheaders():
            if k.lower() not in ("transfer-encoding", "content-length", "connection"):
                self.send_header(k, v)
        self.send_header("Connection", "close")
        self.end_headers()
        out = bytearray()
        first = None
        while True:
            chunk = r.read1(8192) if hasattr(r, "read1") else r.read(8192)
            if not chunk:
                break
            if first is None:
                first = time.time() - t0
            out += chunk
            try:
                self.wfile.write(chunk)
                self.wfile.flush()
            except Exception:
                break
        rec = {"path": self.path, "method": self.command, "status": r.status, "first_byte_s": first, "total_s": time.time() - t0,
               "request": body.decode("utf8", "replace"), "response": out.decode("utf8", "replace")}
        with open(OUT, "a", encoding="utf8") as f:
            f.write(json.dumps(rec) + "\n")

    do_GET = do_POST = _go


ThreadingHTTPServer(("127.0.0.1", LISTEN), H).serve_forever()
