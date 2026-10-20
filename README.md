# Open S85

An open-source, cross-platform replacement for the Royal Kludge configuration software, for the
**RK S85** keyboard. Runs on **Windows, Linux and macOS**.

> Unofficial. Not affiliated with or endorsed by Royal Kludge. Use at your own risk.

## Features

* **Key mapping** on the Default and FN1 layers (FN2 and Tap too), with media keys and modifiers.
  Refuses to remove your last Fn key.
* **Lighting**: all 17 built-in effects, brightness, speed, colour, custom RGB colour, colour mixing.
* **Screen**: upload any image or animated GIF (resized and cropped for you), sync the clock.
* **Wireless sleep time.**
* **Profiles**, stored locally (the keyboard cannot report its key mapping, so the app keeps it).
* A command-line tool, `s85`, for scripting.

Not supported (yet): macros, the PC-driven "real-time" and music lighting effects, per-key colours.

## Install

Download a build from the [Actions](../../actions) tab, or build from source:

```sh
cargo build --release
# GUI:  target/release/open-s85
# CLI:  target/release/s85
```

The keyboard must be connected **by USB cable** (wired mode). Close the official app first: it
polls the keyboard and will also re-send its own saved key mapping.

### Linux

Allow non-root access, then replug the keyboard:

```sh
sudo cp packaging/linux/99-rk-s85.rules /etc/udev/rules.d/
sudo udevadm control --reload && sudo udevadm trigger
```

Build dependencies (Debian/Ubuntu): `libudev-dev libgtk-3-dev libxkbcommon-dev libwayland-dev`.

### macOS

If macOS asks for **Input Monitoring** permission, allow it for the app (or your terminal).

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

See [docs/PROTOCOL.md](docs/PROTOCOL.md). The protocol was worked out from USB captures and by
studying how the official app communicates with the keyboard, solely for interoperability. This
repository contains no code, images or other assets from the official software.

## License

MIT
