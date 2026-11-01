"""Owned loopback-only TLS/CONNECT witness, Python stdlib; no external relay."""
import argparse
import json
import select
import socket
import ssl
import sys
import threading
import time
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--certificate", choices=("leaf.pem", "expired.pem", "host-only.pem"), default="leaf.pem")
parser.add_argument("--proxy-mode", choices=("tunnel", "deny", "hold", "gate", "oversize"), default="tunnel")
args = parser.parse_args()
root = Path(__file__).parent
stop = threading.Event()
release = threading.Event()
output_lock = threading.Lock()
lock = threading.Lock()
receipt = {"tls_connections": 0, "tls_failures": 0, "sni": [], "requests": [],
           "proxy_connections": 0, "connect": [], "plain_connections": 0}
workers = []
deadline = time.monotonic() + 60


COUNTER = object()


def record(key, value=COUNTER):
    with lock:
        if value is COUNTER:
            receipt[key] += 1
        else:
            receipt[key].append(value)


def emit(value):
    with output_lock:
        print(json.dumps(value), flush=True)


def listener():
    sock = socket.socket()
    sock.bind(("127.0.0.1", 0))
    sock.listen(32)
    sock.settimeout(0.05)
    return sock


tls_listener, proxy_listener, plain_listener = listener(), listener(), listener()
tls_port, proxy_port, plain_port = [s.getsockname()[1] for s in (tls_listener, proxy_listener, plain_listener)]
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.minimum_version = ssl.TLSVersion.TLSv1_2
context.load_cert_chain(root / args.certificate, root / "PUBLIC-TEST-ONLY-leaf-key.pem")
context.set_servername_callback(lambda sock, name, ctx: record("sni", name))


def read_head(sock):
    data = bytearray()
    until = time.monotonic() + 3
    while not stop.is_set() and time.monotonic() < until:
        try:
            chunk = sock.recv(1024)
        except socket.timeout:
            continue
        if not chunk:
            return bytes(data)
        data.extend(chunk)
        if len(data) > 32768:
            raise ValueError("fixture input bound")
        end = data.find(b"\r\n\r\n")
        if end >= 0:
            return bytes(data)
    raise TimeoutError("fixture header deadline")


def tls_handler(raw):
    record("tls_connections")
    try:
        raw.settimeout(1)
        with context.wrap_socket(raw, server_side=True) as sock:
            sock.settimeout(0.05)
            data = bytearray(read_head(sock))
            head, _, _ = bytes(data).partition(b"\r\n\r\n")
            size = next((int(line.split(b":", 1)[1]) for line in head.split(b"\r\n")
                         if line.lower().startswith(b"content-length:")), 0)
            if size > 16384:
                raise ValueError("fixture body bound")
            expected = len(head) + 4 + size
            until = time.monotonic() + 3
            while len(data) < expected and not stop.is_set() and time.monotonic() < until:
                try:
                    chunk = sock.recv(1024)
                except socket.timeout:
                    continue
                if not chunk:
                    break
                data.extend(chunk)
            record("requests", bytes(data).decode("ascii"))
            if data.startswith(b"GET /downgrade "):
                response = f"HTTP/1.1 302 Redirect\r\nLocation: http://127.0.0.1:{plain_port}/target?q=fixture-query\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".encode()
            elif data.startswith(b"GET /redirect "):
                response = b"HTTP/1.1 307 Redirect\r\nLocation: /search\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            else:
                response = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\ntls"
            sock.sendall(response)
    except (ssl.SSLError, OSError, TimeoutError, ValueError):
        record("tls_failures")
    finally:
        raw.close()


def proxy_handler(client):
    record("proxy_connections")
    try:
        client.settimeout(0.05)
        head = read_head(client)
        record("connect", head.decode("ascii"))
        emit({"connect_seen": True})
        # Fixed local witness authorities only. The hostname case proves remote-DNS CONNECT
        # behavior without turning this fixture into an arbitrary relay.
        allowed = (
            f"CONNECT 127.0.0.1:{tls_port} HTTP/1.1\r\n".encode(),
            f"CONNECT search-fixture.invalid:{tls_port} HTTP/1.1\r\n".encode(),
        )
        if not head.startswith(allowed):
            client.sendall(b"HTTP/1.1 403 Forbidden\r\n\r\n")
            return
        if args.proxy_mode == "deny":
            client.sendall(b"HTTP/1.1 407 Proxy Authentication Required\r\n\r\n")
            return
        if args.proxy_mode == "oversize":
            client.sendall(b"HTTP/1.1 200 OK\r\nX-Fixture: " + b"x" * 65536 + b"\r\n\r\n")
            return
        if args.proxy_mode == "hold":
            while not stop.is_set() and time.monotonic() < deadline:
                try:
                    if not client.recv(1):
                        emit({"proxy_closed": True})
                        return
                except socket.timeout:
                    pass
            return
        if args.proxy_mode == "gate":
            while not release.wait(0.05) and not stop.is_set() and time.monotonic() < deadline:
                pass
            if stop.is_set() or not release.is_set():
                return
        # Literal fixture authority only; never relay arbitrary hosts or ports.
        with socket.create_connection(("127.0.0.1", tls_port), timeout=1) as target:
            client.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            target.settimeout(0.05)
            while not stop.is_set() and time.monotonic() < deadline:
                ready, _, _ = select.select([client, target], [], [], 0.05)
                for source in ready:
                    data = source.recv(16384)
                    if not data:
                        return
                    (target if source is client else client).sendall(data)
    except ConnectionResetError:
        emit({"proxy_closed": True})
    except (OSError, TimeoutError, ValueError):
        pass
    finally:
        client.close()


def accept_loop(sock, handler, counter):
    while not stop.is_set() and time.monotonic() < deadline:
        try:
            client, _ = sock.accept()
        except socket.timeout:
            continue
        if handler is None:
            record(counter)
            client.close()
        else:
            worker = threading.Thread(target=handler, args=(client,), daemon=True)
            workers.append(worker)
            worker.start()
    # Terminal queue drain makes negative receipts independent of worker scheduling.
    sock.setblocking(False)
    while True:
        try:
            client, _ = sock.accept()
        except BlockingIOError:
            break
        record(counter)
        client.close()
    sock.close()


loops = [threading.Thread(target=accept_loop, args=(s, h, c), daemon=True)
         for s, h, c in ((tls_listener, tls_handler, "tls_connections"),
                         (proxy_listener, proxy_handler, "proxy_connections"),
                         (plain_listener, None, "plain_connections"))]
for loop in loops:
    loop.start()
emit({"tls_port": tls_port, "proxy_port": proxy_port, "plain_port": plain_port})
for line in sys.stdin:
    if line.strip() == "allow":
        release.set()
    else:
        break
stop.set()
for worker in loops:
    worker.join(timeout=2)
for worker in workers:
    worker.join(timeout=2)
if any(worker.is_alive() for worker in loops + workers):
    raise RuntimeError("fixture cleanup incomplete")
emit(receipt)