# Changelog

## 0.2.0 — 2026-10-21

A redesigned desktop app. The keyboard protocol and the command-line tool are unchanged.

### New look
* Dark theme with icons ([Phosphor](https://phosphoricons.com), MIT) throughout.
* Every page and setting has a title, an icon and a one-line explanation; buttons have tooltips.
* Sidebar with a subtitle per page and a connection card (status, device name, Reconnect).
* Status bar shows the last action or error and the active profile.

### Keys
* Layers (Default, FN1, FN2, Tap) are tabs, each with a short explanation.
* Legend for selected, customised and empty keys.
* Card for the selected key with "Factory" (restore that key) and deselect buttons.
* Search box for the function picker (e.g. "volume", "F5").

### Lighting
* Effects are tiles with an icon and a description each.
* Brightness, speed, colour and wireless sleep each have their own card.
* Colour panel like the official app: hue ring (click or drag), R/G/B sliders with number boxes,
  an editable hex code, and eight presets (red, orange, yellow, green, blue, cyan, magenta, white).
* Key legends in the preview stay readable on bright colours.

### Screen
* Drag and drop an image or GIF onto the window, or click the preview to browse.
* Animated GIF preview inside a mock-up of the screen, with play/pause.
* Shows frame count, delay per frame and estimated upload time.
* Fit options (Fill, Fit, Stretch) are tiles explaining what each one does.
* Frame delay override is a slider (10–1000 ms).

### Settings
* "Restore factory key mapping" now asks for confirmation.
* Shows where profiles are stored, and links to the project on GitHub.

## 0.1.0 — 2026-10-20

First release: key mapping on all four layers, the 17 lighting effects with brightness, speed,
colour and colour mixing, wireless sleep time, screen upload of images and GIFs, clock sync,
local profiles, and the `s85` command-line tool.
