"""Extract a keyboard's key geometry from the official app's per-model KB.ini into JSON.

Usage: kbini2json.py "<RK install dir>/Dev/<PID>/KB.ini" layout.json

The official app keeps one Dev/<PID>/KB.ini per model. Each [KEY] entry is
`K<n>=x1,y1,x2,y2, <type>,<vk>,<extra>,<matrix slot>` preceded by a `;label` comment; [FN1] lists the
FN-layer functions. [OPT] holds model options (screen size, speed range, LED options, ...).
This file is not redistributed here; read it from your own installation.
"""
import json
import re
import sys


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    src, out = sys.argv[1:]
    section, label = None, None
    keys, fn1, opt = {}, {}, {}
    for raw in open(src, encoding="latin-1"):
        line = raw.strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
            continue
        if line.startswith(";"):
            label = line[1:]
            continue
        k, _, v = line.partition("=")
        k, v = k.strip(), v.strip()
        if section == "OPT":
            opt[k] = v
        elif section in ("KEY", "FN1") and re.fullmatch(r"K\d+", k):
            parts = [p.strip() for p in v.split(",") if p.strip()]
            n = int(k[1:])
            if section == "KEY":
                x1, y1, x2, y2 = map(int, parts[:4])
                keys[n] = {"id": n, "label": label, "rect": [x1, y1, x2, y2],
                           "code": [int(p, 0) for p in parts[4:7]], "matrix": int(parts[7], 0)}
            else:
                fn1[n] = {"label": label, "code": [int(p, 0) for p in parts]}
        label = None
    with open(out, "w") as f:
        json.dump({"opt": opt, "keys": [keys[i] for i in sorted(keys)], "fn1": fn1}, f, indent=1)
    print(len(keys), "keys,", len(fn1), "FN1 entries")


if __name__ == "__main__":
    main()
