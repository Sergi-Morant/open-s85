#!/usr/bin/env python3
"""Render crates/s85-gui/assets/icon.svg into the PNG sizes, the Windows .ico and the macOS .icns.

Needs `resvg` on PATH (`cargo install resvg`). The outputs are committed, so this only has to be
re-run after editing the SVG:

    python tools/make_icons.py
"""

import struct
import subprocess
import sys
from pathlib import Path

ASSETS = Path(__file__).resolve().parent.parent / "crates" / "s85-gui" / "assets"
PNG_SIZES = [16, 32, 64, 128, 256, 512, 1024]
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]


def render(size: int, out: Path) -> bytes:
    subprocess.run(["resvg", "-w", str(size), "-h", str(size), str(ASSETS / "icon.svg"), str(out)],
                   check=True, stderr=subprocess.DEVNULL)
    return out.read_bytes()


def write_ico(images: list[tuple[int, bytes]], out: Path) -> None:
    """ICO with PNG-compressed entries (supported since Windows Vista)."""
    header = struct.pack("<HHH", 0, 1, len(images))
    offset = len(header) + 16 * len(images)
    entries, blobs = b"", b""
    for size, png in images:
        dim = 0 if size >= 256 else size  # 0 means 256
        entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(png), offset + len(blobs))
        blobs += png
    out.write_bytes(header + entries + blobs)


def write_icns(pngs: dict[int, bytes], out: Path) -> None:
    """ICNS with PNG entries: (type, pixel size) for normal and @2x Retina slots."""
    slots = [("icp4", 16), ("icp5", 32), ("ic11", 32), ("icp6", 64), ("ic12", 64), ("ic07", 128),
             ("ic08", 256), ("ic13", 256), ("ic09", 512), ("ic14", 512), ("ic10", 1024)]
    body = b"".join(t.encode() + struct.pack(">I", 8 + len(pngs[s])) + pngs[s] for t, s in slots)
    out.write_bytes(b"icns" + struct.pack(">I", 8 + len(body)) + body)


def main() -> int:
    icons = ASSETS / "icons"
    icons.mkdir(exist_ok=True)
    pngs = {size: render(size, icons / f"{size}x{size}.png") for size in PNG_SIZES}
    write_icns(pngs, ASSETS / "icon.icns")
    tmp = icons / "_tmp.png"
    ico = [(size, render(size, tmp)) for size in ICO_SIZES]
    tmp.unlink()
    write_ico(ico, ASSETS / "icon.ico")
    print(f"wrote {len(PNG_SIZES)} PNGs to {icons}, icon.ico and icon.icns")
    return 0


if __name__ == "__main__":
    sys.exit(main())
