"""List a device's HID collections with usage page and report sizes (Windows, read-only).

Usage: hidcaps.py [VID_HEX [PID_HEX]]      default: 258a (all RK products)
Requires `pip install hidapi`. The vendor collection whose feature report size matches the official
app's buffers (520 bytes for the S85) is the one to talk to. Opening a collection with zero access
rights is enough to read its capabilities; nothing is sent to the device.
On Linux, read the report descriptor instead: /sys/class/hidraw/hidrawN/device/report_descriptor.
"""
import ctypes
import sys
from ctypes import wintypes

import hid


class HIDP_CAPS(ctypes.Structure):
    _fields_ = [("Usage", ctypes.c_ushort), ("UsagePage", ctypes.c_ushort),
                ("InputReportByteLength", ctypes.c_ushort), ("OutputReportByteLength", ctypes.c_ushort),
                ("FeatureReportByteLength", ctypes.c_ushort), ("Reserved", ctypes.c_ushort * 17),
                ("Rest", ctypes.c_ushort * 10)]


def main():
    vid = int(sys.argv[1], 16) if len(sys.argv) > 1 else 0x258A
    pid = int(sys.argv[2], 16) if len(sys.argv) > 2 else 0
    hiddll = ctypes.WinDLL("hid")
    k32 = ctypes.WinDLL("kernel32", use_last_error=True)
    k32.CreateFileW.restype = wintypes.HANDLE
    for d in hid.enumerate(vid, pid):
        h = k32.CreateFileW(d["path"].decode(), 0, 3, None, 3, 0, None)
        pp, caps = ctypes.c_void_p(), HIDP_CAPS()
        hiddll.HidD_GetPreparsedData(h, ctypes.byref(pp))
        hiddll.HidP_GetCaps(pp, ctypes.byref(caps))
        hiddll.HidD_FreePreparsedData(pp)
        k32.CloseHandle(h)
        print(f"{d['product_id']:04x} if{d['interface_number']} page {caps.UsagePage:#06x} usage {caps.Usage:#04x} "
              f"in={caps.InputReportByteLength} out={caps.OutputReportByteLength} "
              f"feature={caps.FeatureReportByteLength}  {d['product_string']}")


if __name__ == "__main__":
    main()
