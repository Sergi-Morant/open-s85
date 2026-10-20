"""Decode RK S85 HID feature traffic from a USBPcap capture into a timeline of commands.

Usage: decode.py CAPTURE.pcapng [--dump DIR] [--bus N --dev N]

Reassembles multi-chunk transfers (09 <cmd> <arg> 00 <chunks> <idx> <len LE> <data>) and prints one
line per transfer. Repeated all-zero 0x08 frames (the official app's idle stream) are collapsed.
With --dump, each transfer's payload is written to DIR/<n>_<cmd>_<arg>.bin for diffing.
Screen image chunks (0x0C) are skipped; use screen.py for those.
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import add_device_args, find_keyboard, tshark  # noqa: E402

NAMES = {0x03: "SetMatrix", 0x04: "SetLED", 0x05: "SetMacro", 0x06: "SetGame", 0x08: "RealData",
         0x0A: "LedRgbTab", 0x0B: "ScreenParam", 0x0D: "GIFParam", 0x11: "Reset", 0x12: "SendWeb",
         0x82: "GetPsd", 0x84: "GetLED"}


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("capture")
    ap.add_argument("--dump", help="directory to write each transfer's payload to")
    add_device_args(ap)
    a = ap.parse_args()

    bus, dev = find_keyboard(a.capture, a.bus, a.dev)
    print(f"keyboard at bus {bus} device {dev}")
    rows = tshark(a.capture, f"usb.bus_id=={bus} && usb.device_address=={dev} && usb.data_fragment",
                  "frame.number", "frame.time_relative", "usb.data_fragment")

    transfers, cur = [], None
    for num, t, data in rows:
        b = bytes.fromhex(data.replace(":", ""))
        if len(b) < 8 or b[0] != 0x09 or b[1] == 0x0C:
            continue
        cmd, arg, total, idx, ln = b[1], b[2], b[4], b[5], b[6] | b[7] << 8
        if idx == 0 or cur is None or (cur["cmd"], cur["arg"]) != (cmd, arg):
            cur = {"t": float(t), "frame": int(num), "cmd": cmd, "arg": arg, "total": total, "data": bytearray()}
            transfers.append(cur)
        cur["data"] += b[8:8 + ln]

    idle = False
    for n, x in enumerate(transfers):
        d = bytes(x["data"])
        if x["cmd"] == 0x08 and not any(d):
            if not idle:
                print(f"{x['t']:8.2f}  RealData (all zero, repeating...)")
            idle = True
            continue
        idle = False
        name = NAMES.get(x["cmd"], f"cmd{x['cmd']:02x}")
        more = " ..." if len(d.rstrip(b"\0")) > 24 else ""
        print(f"{x['t']:8.2f}  #{n:<4} {name:<11} arg={x['arg']:<3} chunks={x['total']:<2} "
              f"len={len(d):<5} head={d[:24].hex(' ')}{more}")
        if a.dump:
            os.makedirs(a.dump, exist_ok=True)
            with open(os.path.join(a.dump, f"{n:04d}_{x['cmd']:02x}_{x['arg']}.bin"), "wb") as f:
                f.write(d)


if __name__ == "__main__":
    main()
