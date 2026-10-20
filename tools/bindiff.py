"""Byte-level diffs between consecutive payloads of one command dumped by `decode.py --dump`.

Usage: bindiff.py DUMP_DIR CMD_HEX      e.g.  bindiff.py dump 04
Payloads are compared only with the previous one of the same command *and* argument (e.g. the same
key layer). Change exactly one setting between writes and the diff shows which offset holds it.
"""
import glob
import os
import sys


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    d, cmd = sys.argv[1], sys.argv[2].lower().zfill(2)
    prev = {}
    for f in sorted(glob.glob(os.path.join(d, f"*_{cmd}_*.bin"))):
        b = open(f, "rb").read()
        name = os.path.basename(f)
        arg = os.path.splitext(name)[0].split("_")[2]
        p = prev.get(arg)
        if p is None:
            print(f"{name}: first for arg {arg}, {len(b)} bytes")
        else:
            diffs = [(i, p[i], b[i]) for i in range(min(len(p), len(b))) if p[i] != b[i]]
            shown = ", ".join(f"[{i:#05x}] {x:02x}->{y:02x}" for i, x, y in diffs[:24]) or "no change"
            print(f"{name}: {shown}" + (f" ... (+{len(diffs) - 24})" if len(diffs) > 24 else ""))
        prev[arg] = b


if __name__ == "__main__":
    main()
