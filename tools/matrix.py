"""Decode SetMatrix (0x03) payloads dumped by `decode.py --dump`, labelling slots with key names.

Usage: matrix.py layout.json DUMP_FILE.bin [...]
layout.json comes from kbini2json.py. Each slot is a 4-byte big-endian key code (see docs/PROTOCOL.md).
"""
import json
import sys


def main():
    if len(sys.argv) < 3:
        raise SystemExit(__doc__)
    layout = json.load(open(sys.argv[1]))
    by_slot = {k["matrix"]: k for k in layout["keys"]}
    for path in sys.argv[2:]:
        d = open(path, "rb").read()
        print(f"== {path} ({len(d) // 4} slots)")
        for slot in range(len(d) // 4):
            e = d[slot * 4:slot * 4 + 4]
            k = by_slot.get(slot)
            if k is None and not any(e):
                continue
            label = f"{k['label']:<10} vk={k['code'][1]:#04x}" if k else "-"
            print(f"  slot {slot:3d}: {e.hex(' ')}   {label}")


if __name__ == "__main__":
    main()
