"""Summarise screen (TFT) uploads in a USBPcap capture: control packets, chunk ranges, colours, acks.

Usage: screen.py CAPTURE.pcapng [--bus N --dev N]
Upload solid-colour test images with the official app to make the pixel format obvious.
"""
import argparse
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import add_device_args, find_keyboard, tshark  # noqa: E402

COLOURS = {0xF800: "red", 0x07E0: "green", 0x001F: "blue", 0x0000: "black", 0xFFFF: "white"}


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("capture")
    add_device_args(ap)
    a = ap.parse_args()
    bus, dev = find_keyboard(a.capture, a.bus, a.dev)

    rows = tshark(a.capture, f"usb.bus_id=={bus} && usb.device_address=={dev} && usb.data_fragment",
                  "frame.time_relative", "usb.data_fragment")
    chunks = []

    def flush():
        if not chunks:
            return
        frames = collections.Counter(c[3] for _, c in chunks)
        cols = collections.Counter()
        for _, c in chunks:
            body = c[8:8 + (c[6] | c[7] << 8)]
            cols.update(int.from_bytes(body[i:i + 2], "big") for i in range(0, len(body) - 1, 2))
        print(f"  DATA {chunks[0][0]:.2f}-{chunks[-1][0]:.2f}s: {len(chunks)} chunks, "
              f"per frame {dict(frames)}, colours {{{', '.join(f'{COLOURS.get(k, hex(k))}: {v}' for k, v in cols.most_common(4))}}}")
        chunks.clear()

    for t, data in rows:
        b = bytes.fromhex(data.replace(":", ""))
        if len(b) < 8 or b[0] != 0x09:
            continue
        if b[1] == 0x0C:
            chunks.append((float(t), b))
        elif b[1] == 0x0D:
            flush()
            print(f"{float(t):8.2f}  GIFParam {b[8:13].hex(' ')}")
    flush()

    acks = tshark(a.capture, f"usb.bus_id=={bus} && usb.device_address=={dev} && usb.endpoint_address==0x82 "
                             f"&& usb.data_len>0", "usbhid.data")
    print("input reports (acks):", collections.Counter(r[0] for r in acks).most_common(5))


if __name__ == "__main__":
    main()
