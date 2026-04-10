#!/usr/bin/env python3
import RNS
import time

def announce_handler(announce_data, dest_hash, announced_identity):
    print(f"ANNOUNCE: {RNS.hexrep(dest_hash)}")

RNS.Transport.register_announce_handler(announce_handler)
RNS.Reticulum()
print("Listening for announces. Press Ctrl+C to stop.")
try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    pass