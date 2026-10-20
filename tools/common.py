"""Shared helpers for the capture tools: locating tshark and the keyboard in a USBPcap capture."""
import os
import shutil
import subprocess

VID, PIDS = 0x258A, (0x01FA, 0x0193)


def tshark_path():
    """tshark from $TSHARK, PATH, or the default Wireshark install location on Windows."""
    for cand in (os.environ.get("TSHARK"), shutil.which("tshark"),
                 r"C:\Program Files\Wireshark\tshark.exe"):
        if cand and os.path.exists(cand):
            return cand
    raise SystemExit("tshark not found: install Wireshark or set TSHARK=/path/to/tshark")


def tshark(cap, flt, *fields):
    """Run tshark on a capture and return rows of the requested fields."""
    args = [tshark_path(), "-r", cap, "-Y", flt, "-T", "fields", "-E", "separator=|"]
    for f in fields:
        args += ["-e", f]
    out = subprocess.run(args, capture_output=True, text=True, check=True).stdout
    return [line.split("|") for line in out.splitlines()]


def find_keyboard(cap, bus=None, dev=None):
    """(bus, device) of the keyboard: given explicitly, or from its device descriptor in the capture."""
    if bus is not None and dev is not None:
        return str(bus), str(dev)
    pids = " || ".join(f"usb.idProduct=={p:#06x}" for p in PIDS)
    rows = tshark(cap, f"usb.idVendor=={VID:#06x} && ({pids})", "usb.bus_id", "usb.device_address")
    if not rows:
        raise SystemExit("keyboard descriptor not in capture; pass --bus and --dev "
                         "(see 'tshark -r CAP -q -z conv,usb' for addresses)")
    return rows[-1][0], rows[-1][1]


def add_device_args(ap):
    ap.add_argument("--bus", type=int, help="USB bus id of the keyboard (if not auto-detected)")
    ap.add_argument("--dev", type=int, help="USB device address of the keyboard")
