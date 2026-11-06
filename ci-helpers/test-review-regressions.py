#!/usr/bin/env python3
"""Local three-process regressions. Run after cargo build --workspace; takes ~70 seconds."""
import os
import pathlib
import re
import socket
import subprocess
import tempfile
import threading
import time
ROOT = pathlib.Path(__file__).resolve().parents[1]
TEMP = tempfile.TemporaryDirectory(prefix='duotunnel-regressions-')
D = pathlib.Path(TEMP.name)
procs = []
logs = []
stop = threading.Event()

def port(kind=socket.SOCK_STREAM):
    with socket.socket(type=kind) as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]

def receive_until(sock, delimiter):
    data = bytearray()
    while delimiter not in data:
        chunk = sock.recv(4096)
        if not chunk:
            break
        data.extend(chunk)
        if len(data) > 65536:
            raise RuntimeError("unexpectedly large response headers")
    return bytes(data)


def backend():
    s = socket.socket()
    s.bind(('127.0.0.1', 0))
    s.listen()
    s.settimeout(0.3)
    return s
banner = backend()
http = backend()
banner_hits = []

def banner_run():
    while not stop.is_set():
        try:
            c, _ = banner.accept()
        except socket.timeout:
            continue
        with c:
            banner_hits.append(1)
            c.sendall(b'220 test service ready\r\n')

def http_conn(c):
    with c:
        c.settimeout(3)
        try:
            data = receive_until(c, b'\r\n\r\n')
            if b'/idle ' in data:
                c.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n')
                stop.wait(5)
            elif b'/stream ' in data:
                c.sendall(b'HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n')
                for i in range(70):
                    body = f'data: {i}\n\n'.encode()
                    c.sendall(f'{len(body):x}\r\n'.encode() + body + b'\r\n')
                    if stop.wait(1):
                        return
                c.sendall(b'0\r\n\r\n')
            else:
                c.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok')
        except (OSError, ConnectionError):
            pass

def http_run():
    while not stop.is_set():
        try:
            c, _ = http.accept()
        except socket.timeout:
            continue
        threading.Thread(target=http_conn, args=(c,), daemon=True).start()
for f in [banner_run, http_run]:
    threading.Thread(target=f, daemon=True).start()
watch = port()
tunnel = port(socket.SOCK_DGRAM)
tcp = port()
h1 = port()
metrics = port()
D.joinpath('routing.yaml').write_text(f'''server_egress_upstream: {{}}
tunnel_management:
  server_ingress_routing:
    listeners:
      - port: {tcp}
        mode: tcp
        client_group: review
        proxy_name: banner
      - port: {h1}
        mode: http
        vhost:
          - match_host: review.test
            client_group: review
            proxy_name: web
  client_configs:
    groups:
      review:
        config_version: v1
        upstreams:
          banner:
            servers:
              - address: "127.0.0.1:{banner.getsockname()[1]}"
                resolve: false
            lb_policy: round_robin
          web:
            servers:
              - address: "http://127.0.0.1:{http.getsockname()[1]}"
                resolve: false
            lb_policy: round_robin
''')
D.joinpath('ctld.yaml').write_text(f'''database_url: "sqlite://{D}/review.db?mode=rwc"
watch_addr: "127.0.0.1:{watch}"
admin_socket: "{D}/admin.sock"
log_level: info
config:
  sources:
    - type: yaml
      path: "{D}/routing.yaml"
      priority: 10
    - type: sqlite
      database_url: "sqlite://{D}/review.db?mode=rwc"
      priority: 100
''')
D.joinpath('server.yaml').write_text(f'''server:
  tunnel_port: {tunnel}
  metrics_port: {metrics}
  log_level: debug
  accept_workers: 1
''')

def start(name, extra=()):
    l = D.joinpath(name + '.log').open('w')
    logs.append(l)
    p = subprocess.Popen([str(ROOT / 'target/debug' / ('duotunnel-' + name)), '--config', str(D / (name + '.yaml'))] + list(extra), cwd=D, stdout=l, stderr=subprocess.STDOUT, env={**os.environ, 'TOKIO_WORKER_THREADS': '2'})
    procs.append(p)
    return p
try:
    start('ctld')
    for _ in range(100):
        if D.joinpath('admin.sock').exists():
            break
        if procs[-1].poll() is not None:
            raise RuntimeError('ctld startup failed')
        time.sleep(0.1)
    result = subprocess.run([str(ROOT / 'target/debug/duotunnel-ctld'), '--config', str(D / 'ctld.yaml'), 'client', 'create', 'review'], cwd=D, capture_output=True, text=True, timeout=10)
    token = re.search('dt_[A-Za-z0-9_-]{43}', result.stdout)
    if not token:
        raise RuntimeError('temporary token creation failed: ' + result.stderr)
    D.joinpath('client.yaml').write_text(f'''log_level: debug
tls_skip_verify: true
server_addr: "127.0.0.1"
server_port: {tunnel}
auth_token: "{token.group()}"
proxy_buffers:
  http_response_idle_timeout_ms: 1500
quic:
  connections: 1
''')
    start('server', ['--ctld-addr', f'127.0.0.1:{watch}'])
    start('client')
    for _ in range(150):
        try:
            with socket.create_connection(('127.0.0.1', h1), timeout=0.3) as c:
                c.sendall(b'GET / HTTP/1.1\r\nHost: review.test\r\nConnection: close\r\n\r\n')
                if b'200 OK' in receive_until(c, b'\r\n\r\n'):
                    break
        except OSError:
            pass
        time.sleep(0.1)
    else:
        raise RuntimeError('HTTP tunnel readiness failed')
    print('Baseline: normal HTTP request succeeded through ctld/server/client', flush=True)
    with socket.create_connection(('127.0.0.1', tcp), timeout=8) as c:
        t = time.monotonic()
        data = receive_until(c, b'\r\n')
        elapsed = time.monotonic() - t
    print(f'TCP server-first: data={data!r}, received after {elapsed:.2f}s, backend connections={len(banner_hits)}', flush=True)
    assert data == b'220 test service ready\r\n' and len(banner_hits) == 1
    with socket.create_connection(('127.0.0.1', h1), timeout=4) as c:
        c.sendall(b'GET /idle HTTP/1.1\r\nHost: review.test\r\nConnection: close\r\n\r\n')
        t = time.monotonic()
        while c.recv(4096):
            pass
        elapsed = time.monotonic() - t
        assert 1 < elapsed < 3, elapsed
        print(f'HTTP stalled body closed after {elapsed:.2f}s', flush=True)
    with socket.create_connection(('127.0.0.1', h1), timeout=75) as c:
        c.sendall(b'GET /stream HTTP/1.1\r\nHost: review.test\r\nConnection: close\r\n\r\n')
        t = time.monotonic()
        chunks = []
        while True:
            x = c.recv(4096)
            if not x:
                break
            chunks.append(x)
        elapsed = time.monotonic() - t
        data = b''.join(chunks)
    events = data.count(b'data: ')
    final_chunk = data.endswith(b'0\r\n\r\n')
    print(f"HTTP active stream: closed after {elapsed:.2f}s, events={events}, final_chunk={final_chunk}", flush=True)
    assert elapsed >= 69 and data.count(b'data: ') == 70 and data.endswith(b'0\r\n\r\n')
finally:
    stop.set()
    for p in reversed(procs):
        if p.poll() is None:
            p.terminate()
    for p in procs:
        try:
            p.wait(timeout=3)
        except subprocess.TimeoutExpired:
            p.kill()
            p.wait()
    for l in logs:
        l.close()
    banner.close()
    http.close()
    TEMP.cleanup()
