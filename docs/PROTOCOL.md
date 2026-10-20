# RK S85 USB protocol

Observed from the keyboard's USB traffic and from studying how the official Windows app talks to it,
for interoperability. Everything here was verified against real hardware (RK S85 ISO, `258a:01fa`,
wired mode) unless marked *unverified*.

## Transport

* USB HID, vendor collection **usage page `0xFF02`, usage `0x02`** on interface 1.
* Host → keyboard: **feature reports of 520 bytes, report ID `0x09`**.
* Keyboard → host: 8-byte input reports on the same collection (acknowledgements).

### Generic command header

```
09 <cmd> <arg> 00 <chunks> <chunk index> <len lo> <len hi> <up to 512 bytes of data>
```

`<arg>` is command specific (a layer number, a group number…). Payloads up to 512 bytes fit in one
chunk. To **read**, the host sends the command with bit 7 set (`cmd | 0x80`) and then fetches the
feature report; the reply repeats the header and carries the data.

| cmd | purpose | arg | payload |
|---|---|---|---|
| `0x03` | key mapping, one layer | layer (0 Default, 1 FN1, 2 FN2, 3 Tap) | 126 × 4 bytes |
| `0x04` | config block (lighting, sleep, …); `0x84` reads it | 0 | 128 bytes |
| `0x06` | per-key colours | group | 3 × 126 bytes (all R, all G, all B) *unverified* |
| `0x08` | live per-key colours (streamed) | 0 | 3 × 126 bytes *unverified* |
| `0x0A` | effect palettes | 0 | 20 modes × 7 colours × RGB = 420 bytes |
| `0x0B` | screen clock | 0 | 14 bytes |
| `0x0C` | screen image data | — | see below |
| `0x0D` | screen animation start/finish | 0 | 5 bytes |
| `0x11` | factory reset | 0 | 1 byte *unverified, not used* |

## Key mapping (`0x03`)

Each layer is 126 matrix slots of a 4-byte big-endian code. The keyboard cannot report its mapping,
so a layer is always written whole.

| code | meaning |
|---|---|
| `00 00 00 hh` | HID keyboard usage `hh` (A = `04`, Esc = `29`, …) |
| `00 mm 00 00` | modifier(s): LCtrl `01`, LShift `02`, LAlt `04`, LWin `08`, RCtrl `10`, RShift `20`, RAlt `40`, RWin `80` |
| `00 mm 00 hh` | key with modifiers held |
| `02 00 uu uu` | consumer (media) usage `uuuu` |
| `07 00 00 nn` | built-in keyboard function |
| `08 …` | other built-in functions |
| `0D 00 00 00` | Fn |
| `00 00 00 00` | none (on FN layers: no function) |

The factory Default and FN1 layers are in `crates/s85-protocol/src/layout_data.rs`.

## Config block (`0x04` / `0x84`)

128 bytes ending in `5A A5`. Always read, modify and write back; unknown bytes must be preserved.

| offset | meaning |
|---|---|
| `0x03` | debounce |
| `0x09` | 1 when the per-key ("self-define") mode is active |
| `0x0A` | effect (0 = off, 1 = Fixed on, 2 = Respire, 3 = Rainbow, … 17 = Blossoming) |
| `0x18` | wireless sleep time in minutes (10, 20, 40, 60; 0 = never) |
| `0x38 + 2·mode` | brightness 0–20 for that mode |
| `0x39 + 2·mode` | `speed << 4 \| colour`; speed 0–5, colour = palette slot 0–6 or 7 = colour mixing |

## Palettes (`0x0A`)

20 entries (one per effect number) of 7 RGB colours each. Default per entry: red, green, blue,
yellow, magenta, cyan, white. A custom colour is written to slot 0 and selected with colour index 0.

## Clock (`0x0B`)

`00 00 00 <year LE16> <month> <day> <hour> <minute> <second> <weekday, 0 = Sunday> 00 00 00`

## Screen (320 × 172, RGB565)

1. Start: `0x0D`, payload `[0x40 | slot] [frames LE16] [20 LE16]`.
2. For each frame, 215 chunks of 512 bytes, RGB565 **big-endian**, row-major from the top-left, with
   their own header `09 0C 00 <frame> <idx lo> <idx hi> 00 02`. The keyboard acknowledges each chunk
   with the input report `09 0A 06 01 <idx> 00 00 00`; resend on timeout.
3. Finish: `0x0D`, payload `[slot] [frames LE16] [delay ms LE16]`.

At most 60 frames; the keyboard takes ~72 ms per chunk (~15 s per frame).
