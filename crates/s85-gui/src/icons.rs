//! Phosphor icon font (MIT, see assets/LICENSE-Phosphor.txt) and the glyphs the app uses.
//! The font is added as a fallback, so icons can be mixed freely into any text.

use eframe::egui::{self, FontData, FontFamily};
use eframe::epaint::text::{FontInsert, FontPriority, InsertFontFamily};

pub const KEYBOARD: &str = "\u{E2D8}";
pub const LIGHTBULB: &str = "\u{E2DC}";
pub const MONITOR: &str = "\u{E32E}";
pub const GEAR: &str = "\u{E270}";
pub const FILE_IMAGE: &str = "\u{EA24}";
pub const UPLOAD: &str = "\u{E4C0}";
pub const CLOCK: &str = "\u{E19A}";
pub const ARROWS_OUT: &str = "\u{E0A2}";
pub const FRAME_CORNERS: &str = "\u{E626}";
pub const CROP: &str = "\u{E1D4}";
pub const PLUS: &str = "\u{E3D4}";
pub const TRASH: &str = "\u{E4A6}";
pub const PENCIL: &str = "\u{E3B4}";
pub const CHECK: &str = "\u{E182}";
pub const RESTORE: &str = "\u{E038}";
pub const REFRESH: &str = "\u{E094}";
pub const PLUGS_CONNECTED: &str = "\u{EB5A}";
pub const PLUGS: &str = "\u{EB56}";
pub const WARNING: &str = "\u{E4E0}";
pub const INFO: &str = "\u{E2CE}";
pub const SUN: &str = "\u{E472}";
pub const GAUGE: &str = "\u{E628}";
pub const MOON: &str = "\u{E330}";
pub const PALETTE: &str = "\u{E6C8}";
pub const SWATCHES: &str = "\u{E5B8}";
pub const STACK: &str = "\u{E466}";
pub const USER: &str = "\u{E4C2}";
pub const USERS: &str = "\u{E4D6}";
pub const FILM_STRIP: &str = "\u{E792}";
pub const TIMER: &str = "\u{E492}";
pub const HOURGLASS: &str = "\u{E2B2}";
pub const PLAY: &str = "\u{E3D0}";
pub const PAUSE: &str = "\u{E39E}";
pub const GITHUB: &str = "\u{E576}";
pub const SHIELD_WARNING: &str = "\u{E412}";
pub const FLOPPY: &str = "\u{E248}";
pub const CURSOR_CLICK: &str = "\u{E7C8}";
pub const MAGNIFIER: &str = "\u{E30C}";
pub const X: &str = "\u{E4F6}";

// Key picker categories.
pub const HASH: &str = "\u{E2A2}";
pub const TEXT_AA: &str = "\u{E6EE}";
pub const COMMAND: &str = "\u{E1C4}";
pub const FUNCTION: &str = "\u{EBE4}";
pub const ARROWS_DOWN_UP: &str = "\u{E098}";
pub const CALCULATOR: &str = "\u{E538}";
pub const MUSIC_NOTES: &str = "\u{E340}";
pub const DOTS_THREE: &str = "\u{E1FE}";

// Layers.
pub const ARROW_FAT_UP: &str = "\u{E52E}";
pub const ARROW_FAT_LINES_UP: &str = "\u{E52A}";
pub const HAND_TAP: &str = "\u{EC90}";

// Lighting effects.
pub const POWER: &str = "\u{E3DA}";
pub const CIRCLE: &str = "\u{E18A}";
pub const PULSE: &str = "\u{E000}";
pub const RAINBOW: &str = "\u{E598}";
pub const LIGHTNING: &str = "\u{E2DE}";
pub const DROP: &str = "\u{E210}";
pub const SPIRAL: &str = "\u{E9FA}";
pub const RADIO_BUTTON: &str = "\u{EB08}";
pub const STAR: &str = "\u{E46A}";
pub const CIRCLE_HALF: &str = "\u{E18C}";
pub const WAVE_TRIANGLE: &str = "\u{EAA0}";
pub const BROADCAST: &str = "\u{E0F2}";
pub const WAVE_SINE: &str = "\u{EA9A}";
pub const SCAN: &str = "\u{EBB6}";
pub const FAN: &str = "\u{E9F2}";
pub const WAVES: &str = "\u{E6DE}";
pub const FLOWER: &str = "\u{E75E}";

pub fn install(ctx: &egui::Context) {
    let data = FontData::from_static(include_bytes!("../assets/Phosphor.ttf"));
    let families = [FontFamily::Proportional, FontFamily::Monospace]
        .into_iter()
        .map(|family| InsertFontFamily { family, priority: FontPriority::Lowest })
        .collect();
    ctx.add_font(FontInsert::new("phosphor", data, families));
}
