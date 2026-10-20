# How the RK S85 protocol was reverse engineered

This is the method behind [PROTOCOL.md](PROTOCOL.md), written so a person or an AI agent can
reproduce it, verify the open questions, or extend it to another Royal Kludge keyboard. It took one
working session on Windows with a wired RK S85 ISO (`258a:01fa`) and the official "RK Keyboard
Software" (`DeviceDriver.exe`, a 32-bit MFC program).

## Ground rules

* **Interoperability only.** The goal is a program that talks to the keyboard you own. Read the
  official software to learn the *protocol*; do not copy its code, images or data files into this
  repository. Decompiled code, `DeviceDriver.exe`, `KB.ini` and the RK artwork stay on your machine.
* **Only send what you have seen.** Every command this project sends was first observed in a USB
  capture of the official app. Guessing commands at a keyboard can trigger firmware-update or
  bootloader modes.
* **Read before you write.** Configuration that the keyboard can report (the 128-byte config block)
  is always read, modified and written back, so unknown bytes are preserved.
* **Keep a way back.** Before changing key mappings, capture the factory layers (see step 4) so you
  can always restore them.

## Overview

1. Identify the device and its HID collections.
2. Capture USB traffic while the official app changes **one setting at a time**.
3. Decode the captures and diff consecutive writes to find which bytes hold which setting.
4. Read the official app's code to name the commands, learn the packet structure and find fields
   the captures did not exercise.
5. Reimplement, check generated packets byte-for-byte against captures in unit tests, then confirm on
   the hardware with safe, reversible writes and a human looking at the keyboard.

Steps 2–4 feed each other: the code tells you what to capture next, and captures settle what the
decompiled code leaves ambiguous.

## 1. Identify the device

* Windows Device Manager / `Get-PnpDevice` shows the keyboard as VID `258A` (SinoWealth), PID `01FA`.
  A second device, `258A:0150`, is the 2.4 GHz dongle.
* The official app ships one `Dev/<PID>/KB.ini` per model. For the S85 it says
  `siScreen=320,172`, `RGBScreen=1`, `MaxScreenFrame=60`, `DataUnitSize=512`, `SpeedParam=0x010500`,
  the per-key geometry and matrix slot for every key, and the FN1 layer defaults.
  `Cfg.ini` maps PIDs to model names.
* List the HID collections and their report sizes:

  ```sh
  pip install hidapi
  python tools/hidcaps.py 258a 01fa
  ```

  The S85 has four vendor collections on interface 1. The one with a **520-byte feature report**
  (usage page `0xFF02`, usage `0x02`) matches `DataUnitSize=512` + an 8-byte header: that is the
  channel for everything except the keyboard input itself.

## 2. Capture USB traffic

### Setup (Windows)

1. Install Wireshark and USBPcap (`winget install WiresharkFoundation.Wireshark desowin.USBPcap`).
2. **Reboot** — the USBPcap filter driver only attaches after a restart.
3. If Wireshark shows no `USBPcapN` interfaces, copy `C:\Program Files\USBPcap\USBPcapCMD.exe` into
   `C:\Program Files\Wireshark\extcap\` (needs admin) and restart Wireshark. The "Npcap not
   installed" warning is about network capture and can be ignored.
4. Use the gear next to each `USBPcapN` interface to find the one listing the keyboard, or capture
   on all of them. Connect the keyboard **by cable**.

On Linux you can capture with `usbmon` (`sudo modprobe usbmon`, then the `usbmonN` interfaces in
Wireshark) while the official app runs in a Windows VM with the keyboard passed through.

### Procedure

* Start capturing, open the official app, make **one change, wait ~10 seconds**, make the next.
  The gaps make the decoder's timeline easy to segment. Note the order of your actions.
* Prefer extreme or distinctive values: brightness all the way down and up, custom colour
  `R=10 G=20 B=30` (easy to spot as `0a 14 1e`), solid-colour test images for the screen.
* For the screen, upload a **solid red** image and a **2-frame red/blue GIF** at a known delay.
  Red as RGB565 is `F800`, blue `001F`, so the byte order is obvious.
* For key mapping, remap a key and then set it back: the app re-sends the **whole layer** each time,
  so the second write is the factory layer.
* Close the official app afterwards: while running it polls the keyboard every ~0.8 s and can make
  other programs' transfers time out.

Captures used for this project: screen upload, a lighting/keymap session, and a Default-layer
remap. They contain only your own keyboard traffic but are not committed (`*.pcapng` is ignored).

## 3. Decode and diff

```sh
python tools/decode.py features.pcapng --dump dump/   # timeline of every command + payload files
python tools/bindiff.py dump 04                       # what changed between config writes
python tools/bindiff.py dump 0a                       # ... between palette writes
python tools/screen.py upload.pcapng                  # screen uploads: chunks, colours, acks
python tools/kbini2json.py "C:/Program Files (x86)/RK Keyboard Software/Dev/01FA/KB.ini" layout.json
python tools/matrix.py layout.json dump/0004_03_0.bin # key layer, slot by slot with key names
```

`tshark` is found automatically (or set `TSHARK`). If the keyboard's descriptor is not in the
capture, pass `--bus N --dev N` (see `tshark -r CAP -q -z conv,usb`).

What the diffs showed, for example:

| action in the app | diff |
|---|---|
| Fixed on → OFF → Fixed on | config `[0x0a]` `01→00→01` |
| brightness slider left, then right | config `[0x3a]` `14→00→14` |
| Color Mixing on, off | config `[0x3b]` `10→17→10` |
| wireless sleep changes | config `[0x18]` `14→28→3c` |
| custom colour R=10 G=20 B=30 | palette bytes `0x15..0x17` = `0a 14 1e` |
| Caps Lock → A, then back to Caps Lock | Default layer slot 3: `00000004`, then `00000039` |

## 4. Read the official app

Static analysis explains *why* the bytes look the way they do and names the commands.

### Tooling

* [rea](https://github.com/morluto/rea) (`npm i -g rea-agents`) with Ghidra 12.1.x and JDK 21:
  `rea search <exe> "HidD_" --kind procedures --provider ghidra --snapshot snap`. It works, but on
  Windows each CLI query restarted Ghidra (~2.5 min); use its MCP server to keep a session warm.
* Faster for bulk reading: run Ghidra headless once and export every function as C, then grep.

  ```sh
  analyzeHeadless <proj> rk -import DeviceDriver.exe -overwrite \
      -scriptPath tools/ghidra -postScript ExportDecompiled.java DeviceDriver.c
  ```

  `analyzeHeadless.bat` breaks on paths containing parentheses: copy the exe out of
  `Program Files (x86)` first. Expect ~10 minutes and ~11 MB of output. Keep it private.

### What to look for

1. **Imports** `HidD_SetFeature` / `HidD_GetFeature`. Their callers are the send routine. In this app
   it is one generic function (its log strings call it `CDevG5KB::AccessData`) that builds the
   8-byte header, splits payloads into 512-byte chunks, retries 3× with 50 ms gaps, and for reads
   sends `cmd | 0x80` then calls `GetFeature` and checks the echoed command byte.
2. **Callers of the send routine.** Each passes a constant command byte and length; the nearby
   **debug strings** name them: `SetMatrix`, `SetLED`, `SetMacro`, `SetGame`, `SetLedRgbTab`,
   `SetScreenParam`, `ResetDevice`, `SendWeb`, `SetGIFParam`, `GetPassword`.
3. **The "apply settings" routine** (`ApplySetting` in the logs) shows the write order and what each
   flag writes: key layers then macros; read-modify-write of the config block (it rejects a block
   without the `5A A5` trailer); palettes; per-key colour groups.
4. **The config block fillers** (`FillCfg` in the logs) give the field offsets: debounce, effect,
   per-mode brightness and `speed<<4 | colour`, sleep time, Tap.
5. **`SetGIFParam`** shows the screen start/finish payload packing (`begin<<6 | slot`, frames and
   delay as little-endian u16) — a capture with only small values cannot tell LE from BE.
6. **`GetPrivateProfileIntW` calls** show how `KB.ini` options are interpreted (e.g. `SpeedParam` =
   min, max, default speed).

Parts of the app talk to older RK boards with 65-byte reports (`CDevRKKB`); ignore code using
`0x41`-byte buffers. Key-code conversion happens through C++ virtual calls that decompile poorly —
captures of the factory layers answered that faster.

## 5. Implement and verify

* **Unit tests reproduce captures byte-for-byte** (`cargo test`): the 2-frame red/blue upload,
  the config change the app made on connect, the palette layout, the clock packet, the FN1 header.
* **Hardware checks, safest first:**
  1. read-only: `s85 info`, `s85 light` (reads the config block);
  2. reversible writes: `s85 clock`, `s85 light --mode 2` (a person confirmed "breathing" = Respire,
     which pinned the effect numbering), screen upload of an orientation test image (coloured
     corners + text) to confirm row order and colours;
  3. key layers only after the factory layers were captured, starting with re-writing the captured
     factory FN1 layer.

## Pitfalls we hit

* The official app **re-sends its own saved profile** (all dirty layers) whenever you remap a key,
  overwriting changes made by other tools. Do not mix it with this app.
* The keyboard **cannot report its key mapping or palettes**; the host must keep them.
* On the Default layer, **Fn is `0D 00 00 00`**. Writing a Default layer without it locks you out of
  FN1; the app refuses to.
* FN2 and Tap factory contents were never captured, so this app only writes them once the user
  assigns something there.
* `HidD_SetFeature` times out (`ERROR_SEM_TIMEOUT`) while the official app is polling; close it.
* In the screen protocol each 512-byte chunk is acknowledged by index via an input report; the
  official app resends a chunk when the ack index does not match.

## Open questions

| item | what is known | how to settle it |
|---|---|---|
| per-key colours `0x06` | name, planar R/G/B × 126 by group | capture the Custom Lights page applying a static colour to a few keys; diff |
| real-time stream `0x08` | 378 zero bytes every 0.8 s while the app idles | capture a real-time effect actually running |
| macros `0x05` | blob of offset table + events | capture one tiny macro (one key) assigned to one key |
| FN2 / Tap factory layers | unknown | remap a key on FN2 / Tap and back; the second write is factory |
| config bytes 1, 2, 6–8, 0x16, 0x17, 0x31–0x37 | preserved untouched | change debounce / Tap / other settings in the app one at a time |
| clock byte 0 (`00` then `31` on connect) | unknown | several connects, compare |
| speed direction (is 5 fast or slow?) | range 0–5, default 1 | set 0 and 5 on Respire and watch |
| 2.4 GHz dongle / Bluetooth | not investigated | same captures with the dongle (`258a:0150`) |

## Supporting another RK model

1. Find its PID in `Cfg.ini`, check `Dev/<PID>/KB.ini` (`siScreen`, `ShowScreen`, `LedOpt*`,
   `SpeedParam`) and run `tools/hidcaps.py` to confirm the same 520-byte collection.
2. Capture the factory Default and FN1 layers (remap a key and back on each layer).
3. `tools/kbini2json.py` + `tools/gen_layout_rs.py` → a layout data file for that model.
4. Add the PID to `PIDS` in `crates/s85-protocol/src/lib.rs`, and repeat the lighting captures: effect
   lists and capabilities differ between models.
