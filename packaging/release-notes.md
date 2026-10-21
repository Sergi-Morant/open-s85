
### Downloads
| Platform | Installer | Plain archive |
|---|---|---|
| Windows x86-64 | `open-s85-VERSION-windows-x86_64-setup.exe` | `open-s85-VERSION-windows-x86_64.zip` |
| macOS (Apple Silicon + Intel) | `open-s85-VERSION-macos-universal.dmg` | `open-s85-VERSION-macos-universal.tar.gz` |
| Linux x86-64 (glibc ≥ 2.39, e.g. Ubuntu 24.04+) | — | `open-s85-VERSION-linux-x86_64.tar.gz` |

The archives contain `open-s85` (desktop app), `s85` (CLI), README, CHANGELOG and LICENSE; the
Linux one also has the udev rule.

### Notes
- Connect the keyboard **by USB cable** and close the official RK app first.
- The installers are **not code-signed**, so the first launch shows a warning:
  - **Windows:** "Windows protected your PC" → **More info** → **Run anyway**.
  - **macOS:** "Apple could not verify…" → **System Settings → Privacy & Security** → **Open Anyway**. Allow **Input Monitoring** if asked.
- **Linux:** install `99-rk-s85.rules` (see README) so the app can access the keyboard without root.
- Tested on Windows and macOS with a real RK S85 ISO. The Linux build passes CI but has not yet been tried on hardware — reports welcome.

Not affiliated with or endorsed by Royal Kludge.
