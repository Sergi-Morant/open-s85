# Open S85

An open-source, cross-platform replacement for the Royal Kludge configuration software, for the
**RK S85** keyboard. Runs on **Windows, Linux and macOS**.

> Unofficial. Not affiliated with or endorsed by Royal Kludge. Use at your own risk.

## Features

* **Key mapping** on the Default and FN1 layers (FN2 and Tap too), with media keys and modifiers
  and a searchable function picker. Refuses to remove your last Fn key.
* **Lighting**: all 17 built-in effects, brightness, speed, colour mixing, and a colour panel like
  the official app's (hue ring, R/G/B sliders, hex code, presets).
* **Screen**: drag and drop any image or animated GIF, preview the animation, upload it (resized
  and cropped for you), sync the clock.
* **Wireless sleep time.**
* **Profiles**, stored locally (the keyboard cannot report its key mapping, so the app keeps it).
* A command-line tool, `s85`, for scripting.

Not supported (yet): macros, the PC-driven "real-time" and music lighting effects, per-key colours.

See [CHANGELOG.md](CHANGELOG.md) for what changed in each version.

## Install

Download the latest version from [Releases](../../releases):

| Platform | Download |
|---|---|
| Windows | `…-windows-x86_64-setup.exe` (installer) or `…-windows-x86_64.zip` |
| macOS (Apple Silicon and Intel) | `…-macos-universal.dmg` (open it, drag Open S85 to Applications) or `…-macos-universal.tar.gz` |
| Linux | `…-linux-x86_64.tar.gz` |

The installers are not code-signed, so the first launch shows a warning:

* **Windows:** "Windows protected your PC" → **More info** → **Run anyway**.
* **macOS:** "Apple could not verify…" → **System Settings → Privacy & Security** → **Open Anyway**.

Or build from source:

```sh
cargo build --release
# GUI:  target/release/open-s85
# CLI:  target/release/s85
```

To build the installers yourself, install [cargo-packager](https://github.com/crabnebula-dev/cargo-packager)
(`cargo install cargo-packager --locked`), then run `cargo packager --release` in `crates/s85-gui`
after the build. `.github/workflows/ci.yml` shows the exact steps, including the universal macOS build.

The keyboard must be connected **by USB cable** (wired mode). Close the official app first: it
polls the keyboard and will also re-send its own saved key mapping.

### Linux

Allow non-root access, then replug the keyboard:

```sh
sudo cp packaging/linux/70-rk-s85.rules /etc/udev/rules.d/
sudo udevadm control --reload && sudo udevadm trigger
```

If you installed `99-rk-s85.rules` from 0.2.1 or earlier, it never took effect; delete it
(`sudo rm /etc/udev/rules.d/99-rk-s85.rules`) and install `70-rk-s85.rules` as above.

Build dependencies (Debian/Ubuntu): `libudev-dev libgtk-3-dev libxkbcommon-dev libwayland-dev`.

### macOS

If macOS asks for **Input Monitoring** permission, allow it for the app (or your terminal).

Builds from the Actions tab (not Releases) lose their executable bit in the zip. After unzipping:

```sh
chmod +x open-s85 s85
xattr -d com.apple.quarantine open-s85 s85   # only if macOS refuses to open them
```

## Command line

```sh
s85 info                         # detect the keyboard
s85 light                        # show the current effect
s85 light --mode 2 --speed 3     # Respire, faster
s85 light --mode 1 --rgb ff8800  # Fixed on, orange
s85 modes                        # list effects
s85 screen cat.gif               # upload to the screen (--fit contain|stretch, --delay ms)
s85 clock                        # set the screen clock to local time
```

## How it works

* [docs/PROTOCOL.md](docs/PROTOCOL.md) — the USB protocol.
* [docs/REVERSE-ENGINEERING.md](docs/REVERSE-ENGINEERING.md) — how it was worked out, step by step,
  with the open questions and how to support another RK model.
* [tools/](tools) — the capture decoders and generators used (Python 3, need Wireshark's `tshark`).

The protocol was worked out from USB captures and by studying how the official app communicates
with the keyboard, solely for interoperability. This repository contains no code, images or other
assets from the official software.

## License

MIT. The bundled icon font (`crates/s85-gui/assets/Phosphor.ttf`) is from
[Phosphor Icons](https://phosphoricons.com), also MIT. The app icon is `crates/s85-gui/assets/icon.svg`;
`python tools/make_icons.py` regenerates the PNG and `.ico` files from it.
