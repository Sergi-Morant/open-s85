//! The 4-byte codes the keyboard stores per key (big-endian u32 on the wire).
//!
//! | byte 0 | layout | meaning |
//! |---|---|---|
//! | `00` | `00 <mods> 00 <hid>` | keyboard usage, optionally with modifiers |
//! | `02` | `02 00 <usage u16>` | consumer (media) usage |
//! | `07` | `07 00 00 <n>` | keyboard function (lighting, profiles, ...) |
//! | `08` | `08 ...` | other built-in functions |
//! | `0D` | `0D 00 00 00` | Fn |

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct KeyCode(pub u32);

/// Modifier bits as stored in byte 1.
pub mod mods {
    pub const LCTRL: u8 = 0x01;
    pub const LSHIFT: u8 = 0x02;
    pub const LALT: u8 = 0x04;
    pub const LWIN: u8 = 0x08;
    pub const RCTRL: u8 = 0x10;
    pub const RSHIFT: u8 = 0x20;
    pub const RALT: u8 = 0x40;
    pub const RWIN: u8 = 0x80;
}

impl KeyCode {
    /// No function; on FN layers this means "fall through".
    pub const NONE: KeyCode = KeyCode(0);
    pub const FN: KeyCode = KeyCode(0x0D00_0000);

    /// A HID keyboard usage (page 0x07), e.g. `0x04` = A.
    pub const fn key(hid: u8) -> KeyCode {
        KeyCode(hid as u32)
    }
    /// A modifier-only key, e.g. `mods::LCTRL`.
    pub const fn modifier(m: u8) -> KeyCode {
        KeyCode((m as u32) << 16)
    }
    /// A key with modifiers held, e.g. Ctrl+C.
    pub const fn combo(m: u8, hid: u8) -> KeyCode {
        KeyCode((m as u32) << 16 | hid as u32)
    }
    /// A consumer-page usage (media keys), e.g. `0xE9` = Volume Up.
    pub const fn consumer(usage: u16) -> KeyCode {
        KeyCode(0x0200_0000 | usage as u32)
    }

    pub fn to_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
    pub fn from_bytes(b: [u8; 4]) -> KeyCode {
        KeyCode(u32::from_be_bytes(b))
    }

    /// Human-readable name, falling back to hex.
    pub fn name(self) -> String {
        if let Some(e) = catalog().iter().find(|e| e.code == self) {
            return e.name.to_string();
        }
        let [kind, m, a, b] = self.to_bytes();
        match kind {
            0x00 if a == 0 && m != 0 && b != 0 => {
                let base = KeyCode::key(b).name();
                format!("{}+{}", mod_names(m), base)
            }
            _ => self.to_string(),
        }
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

fn mod_names(m: u8) -> String {
    let names = [
        (mods::LCTRL, "LCtrl"),
        (mods::LSHIFT, "LShift"),
        (mods::LALT, "LAlt"),
        (mods::LWIN, "LWin"),
        (mods::RCTRL, "RCtrl"),
        (mods::RSHIFT, "RShift"),
        (mods::RALT, "RAlt"),
        (mods::RWIN, "RWin"),
    ];
    names.iter().filter(|(b, _)| m & b != 0).map(|(_, n)| *n).collect::<Vec<_>>().join("+")
}

/// Groups shown in the key picker, mirroring the official app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Number,
    Characters,
    Modifier,
    Function,
    Arrow,
    Numpad,
    Media,
    Other,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::Number,
        Category::Characters,
        Category::Modifier,
        Category::Function,
        Category::Arrow,
        Category::Numpad,
        Category::Media,
        Category::Other,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Category::Number => "Number",
            Category::Characters => "Characters",
            Category::Modifier => "Modifier Key",
            Category::Function => "Function Key",
            Category::Arrow => "Arrow Keys",
            Category::Numpad => "Keyboard",
            Category::Media => "Media",
            Category::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CatalogEntry {
    pub code: KeyCode,
    pub name: &'static str,
    pub category: Category,
}

macro_rules! entries {
    ($($cat:ident: [$($name:literal = $code:expr),* $(,)?]),* $(,)?) => {
        &[$($(CatalogEntry { code: $code, name: $name, category: Category::$cat },)*)*]
    };
}

use KeyCode as K;

/// Every assignable function the picker offers.
pub fn catalog() -> &'static [CatalogEntry] {
    CATALOG
}

static CATALOG: &[CatalogEntry] = entries! {
        Number: [
            "1" = K::key(0x1E), "2" = K::key(0x1F), "3" = K::key(0x20), "4" = K::key(0x21), "5" = K::key(0x22),
            "6" = K::key(0x23), "7" = K::key(0x24), "8" = K::key(0x25), "9" = K::key(0x26), "0" = K::key(0x27),
        ],
        Characters: [
            "A" = K::key(0x04), "B" = K::key(0x05), "C" = K::key(0x06), "D" = K::key(0x07), "E" = K::key(0x08),
            "F" = K::key(0x09), "G" = K::key(0x0A), "H" = K::key(0x0B), "I" = K::key(0x0C), "J" = K::key(0x0D),
            "K" = K::key(0x0E), "L" = K::key(0x0F), "M" = K::key(0x10), "N" = K::key(0x11), "O" = K::key(0x12),
            "P" = K::key(0x13), "Q" = K::key(0x14), "R" = K::key(0x15), "S" = K::key(0x16), "T" = K::key(0x17),
            "U" = K::key(0x18), "V" = K::key(0x19), "W" = K::key(0x1A), "X" = K::key(0x1B), "Y" = K::key(0x1C),
            "Z" = K::key(0x1D), "-" = K::key(0x2D), "=" = K::key(0x2E), "[" = K::key(0x2F), "]" = K::key(0x30),
            "\\" = K::key(0x31), ";" = K::key(0x33), "'" = K::key(0x34), "`" = K::key(0x35), "," = K::key(0x36),
            "." = K::key(0x37), "/" = K::key(0x38), "ISO \\" = K::key(0x32), "ISO <>" = K::key(0x64),
        ],
        Modifier: [
            "LCtrl" = K::modifier(mods::LCTRL), "LShift" = K::modifier(mods::LSHIFT),
            "LAlt" = K::modifier(mods::LALT), "LWin" = K::modifier(mods::LWIN),
            "RCtrl" = K::modifier(mods::RCTRL), "RShift" = K::modifier(mods::RSHIFT),
            "RAlt" = K::modifier(mods::RALT), "RWin" = K::modifier(mods::RWIN),
        ],
        Function: [
            "F1" = K::key(0x3A), "F2" = K::key(0x3B), "F3" = K::key(0x3C), "F4" = K::key(0x3D),
            "F5" = K::key(0x3E), "F6" = K::key(0x3F), "F7" = K::key(0x40), "F8" = K::key(0x41),
            "F9" = K::key(0x42), "F10" = K::key(0x43), "F11" = K::key(0x44), "F12" = K::key(0x45),
            "Esc" = K::key(0x29), "Tab" = K::key(0x2B), "Enter" = K::key(0x28), "Space" = K::key(0x2C),
            "Backspace" = K::key(0x2A), "CapsLock" = K::key(0x39), "PrintScreen" = K::key(0x46),
            "ScrollLock" = K::key(0x47), "Pause" = K::key(0x48), "Insert" = K::key(0x49), "Home" = K::key(0x4A),
            "PageUp" = K::key(0x4B), "Delete" = K::key(0x4C), "End" = K::key(0x4D), "PageDown" = K::key(0x4E),
            "Menu" = K::key(0x65),
        ],
        Arrow: [
            "Up" = K::key(0x52), "Down" = K::key(0x51), "Left" = K::key(0x50), "Right" = K::key(0x4F),
        ],
        Numpad: [
            "Num 1" = K::key(0x59), "Num 2" = K::key(0x5A), "Num 3" = K::key(0x5B), "Num 4" = K::key(0x5C),
            "Num 5" = K::key(0x5D), "Num 6" = K::key(0x5E), "Num 7" = K::key(0x5F), "Num 8" = K::key(0x60),
            "Num 9" = K::key(0x61), "Num 0" = K::key(0x62), "Num +" = K::key(0x57), "Num -" = K::key(0x56),
            "Num *" = K::key(0x55), "Num /" = K::key(0x54), "Num ." = K::key(0x63), "Num Enter" = K::key(0x58),
            "NumLock" = K::key(0x53),
        ],
        Media: [
            "Play/Pause" = K::consumer(0xCD), "Next Track" = K::consumer(0xB5), "Prev Track" = K::consumer(0xB6),
            "Stop" = K::consumer(0xB7), "Mute" = K::consumer(0xE2), "Volume Up" = K::consumer(0xE9),
            "Volume Down" = K::consumer(0xEA), "Media Player" = K::consumer(0x183), "Mail" = K::consumer(0x18A),
            "Calculator" = K::consumer(0x192), "My Computer" = K::consumer(0x194), "Browser Home" = K::consumer(0x223),
        ],
        Other: [
            "Fn" = K::FN, "Knob" = KeyCode(0x0700_000D), "None" = K::NONE,
        ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_format() {
        assert_eq!(KeyCode::key(0x39).to_bytes(), [0, 0, 0, 0x39]);
        assert_eq!(KeyCode::modifier(mods::LSHIFT).to_bytes(), [0, 0x02, 0, 0]);
        assert_eq!(KeyCode::consumer(0x194).to_bytes(), [0x02, 0, 0x01, 0x94]);
        assert_eq!(KeyCode::FN.to_bytes(), [0x0D, 0, 0, 0]);
    }

    #[test]
    fn names() {
        assert_eq!(KeyCode::key(0x04).name(), "A");
        assert_eq!(KeyCode::combo(mods::LCTRL, 0x06).name(), "LCtrl+C");
        assert_eq!(KeyCode(0x0800_0000).name(), "0x08000000");
    }
}
