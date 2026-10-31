#!/usr/bin/env python3
"""Small Native Messaging bridge. It forwards extension intervals to Daymark's local Unix socket."""
import json, struct, sys, socket, os

SOCKET = os.path.expanduser('~/Library/Application Support/Daymark/events.sock')
def read_message():
    raw = sys.stdin.buffer.read(4)
    if len(raw) != 4: return None
    length = struct.unpack('=I', raw)[0]
    return json.loads(sys.stdin.buffer.read(length).decode('utf-8'))
def main():
    while True:
        event = read_message()
        if event is None: break
        try:
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
                s.settimeout(1); s.connect(SOCKET); body=json.dumps(event).encode(); s.sendall(struct.pack('=I',len(body))+body)
        except OSError:
            pass
if __name__ == '__main__': main()
