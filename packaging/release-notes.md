
### Downloads
| Platform | File |
|---|---|
| Windows x86-64 | `open-s85-VERSION-windows-x86_64.zip` |
| Linux x86-64 (glibc ≥ 2.39, e.g. Ubuntu 24.04+) | `open-s85-VERSION-linux-x86_64.tar.gz` |
| macOS Apple Silicon | `open-s85-VERSION-macos-arm64.tar.gz` |

Each archive contains `open-s85` (desktop app), `s85` (CLI), README, CHANGELOG and LICENSE; the
Linux one also has the udev rule.

### Notes
- Connect the keyboard **by USB cable** and close the official RK app first.
- **Linux:** install `99-rk-s85.rules` (see README) so the app can access the keyboard without root.
- **macOS:** the binaries are not signed. If macOS blocks them, run `xattr -d com.apple.quarantine open-s85 s85`.
- Tested on Windows and macOS with a real RK S85 ISO. The Linux build passes CI but has not yet been tried on hardware — reports welcome.

Not affiliated with or endorsed by Royal Kludge.
