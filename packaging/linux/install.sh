#!/bin/sh
# Install Open S85 for the current user: binaries, app icon and menu entry.
# Run from the extracted release folder:  ./install.sh   (or ./install.sh --uninstall)
# The udev rule needs root, so it is only printed, not installed.
set -eu

ID=io.github.sergi-morant.open-s85
DIR=$(cd "$(dirname "$0")" && pwd)
BIN=${XDG_BIN_HOME:-$HOME/.local/bin}
DATA=${XDG_DATA_HOME:-$HOME/.local/share}
ICONS=$DATA/icons/hicolor

if [ "${1:-}" = --uninstall ]; then
    rm -f "$BIN/open-s85" "$BIN/s85" "$DATA/applications/$ID.desktop"
    rm -f "$ICONS"/*/apps/"$ID".png
    echo "Removed Open S85. To remove the udev rule: sudo rm /etc/udev/rules.d/70-rk-s85.rules"
    exit 0
fi

install -Dm755 "$DIR/open-s85" "$BIN/open-s85"
install -Dm755 "$DIR/s85" "$BIN/s85"
for png in "$DIR"/icons/*.png; do
    size=$(basename "$png" .png)
    install -Dm644 "$png" "$ICONS/$size/apps/$ID.png"
done
# Absolute Exec path, so the menu entry works even if ~/.local/bin is not on PATH.
mkdir -p "$DATA/applications"
sed "s|^Exec=open-s85|Exec=$BIN/open-s85|" "$DIR/$ID.desktop" > "$DATA/applications/$ID.desktop"
command -v update-desktop-database >/dev/null && update-desktop-database -q "$DATA/applications" || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t "$ICONS" 2>/dev/null || true

echo "Installed open-s85 and s85 to $BIN, with menu entry and icon."
case ":$PATH:" in *":$BIN:"*) ;; *) echo "Note: $BIN is not on your PATH." ;; esac
echo
echo "To use the keyboard without root, install the udev rule once and replug the keyboard:"
echo "  sudo cp '$DIR/70-rk-s85.rules' /etc/udev/rules.d/ && sudo udevadm control --reload && sudo udevadm trigger"
