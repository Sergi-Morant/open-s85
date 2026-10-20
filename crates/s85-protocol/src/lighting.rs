//! Lighting: the 128-byte config block (GetLED 0x84 / SetLED 0x04) and the per-mode palettes
//! (SetLedRgbTab 0x0A).
//!
//! The config block also carries non-lighting settings (sleep, debounce, ...). It is always
//! read, modified and written back, so fields this crate does not understand are preserved.

use crate::device::Device;
use crate::keymap::build_access;
use crate::packet::cmd;
use crate::{Error, Result};

pub const CONFIG_LEN: usize = 128;

/// Built-in effects, numbered as the firmware stores them (= position in the official app's list).
pub const MODES: &[(u8, &str)] = &[
    (1, "Fixed on"),
    (2, "Respire"),
    (3, "Rainbow"),
    (4, "Flash away"),
    (5, "Raindrops"),
    (6, "Rainbow wheel"),
    (7, "Ripples shining"),
    (8, "Stars twinkle"),
    (9, "Shadow disappear"),
    (10, "Retro snake"),
    (11, "Neon stream"),
    (12, "Reaction"),
    (13, "Sine wave"),
    (14, "Retinue scanning"),
    (15, "Rotating windmill"),
    (16, "Colorful waterfall"),
    (17, "Blossoming"),
];
pub const MODE_OFF: u8 = 0;
/// Number of palette entries the firmware keeps (modes 0..=19).
pub const PALETTE_MODES: usize = 20;
pub const PALETTE_COLORS: usize = 7;
pub const MAX_BRIGHTNESS: u8 = 20;
/// The official app offers brightness in steps of 4 (0, 4, ..., 20).
pub const BRIGHTNESS_STEP: u8 = 4;
/// Speed range from the S85's `SpeedParam=0x010500` (min 0, max 5, default 1).
pub const MAX_SPEED: u8 = 5;
pub const DEFAULT_SPEED: u8 = 1;
/// Colour index meaning "colour mixing" (multicolour) instead of a palette entry.
pub const COLOR_MIX: u8 = 7;
/// Values the official app offers for the wireless sleep time, in minutes (0 = never).
pub const SLEEP_MINUTES: [u8; 5] = [10, 20, 40, 60, 0];

/// Which built-in modes support which controls (from the official app's per-model config).
pub fn mode_has_speed(mode: u8) -> bool {
    !matches!(mode, 0 | 1)
}
pub fn mode_has_color(mode: u8) -> bool {
    !matches!(mode, 0 | 3 | 15 | 16 | 17)
}

const OFF_MODE: usize = 10;
const OFF_CUSTOM: usize = 9;
const OFF_SLEEP: usize = 0x18;
const OFF_DEBOUNCE: usize = 3;
const OFF_PER_MODE: usize = 0x38;
const MAGIC: [u8; 2] = [0x5A, 0xA5];

/// The raw 128-byte config block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config(pub [u8; CONFIG_LEN]);

impl Config {
    pub fn read(dev: &Device) -> Result<Config> {
        let r = dev.read_access(cmd::SETTINGS | cmd::READ, 0, CONFIG_LEN)?;
        let mut b = [0u8; CONFIG_LEN];
        b.copy_from_slice(&r[..CONFIG_LEN]);
        if b[CONFIG_LEN - 2..] != MAGIC {
            return Err(Error::Invalid("config block from keyboard has no 5A A5 marker".into()));
        }
        Ok(Config(b))
    }

    pub fn write(&self, dev: &Device) -> Result<()> {
        if self.0[CONFIG_LEN - 2..] != MAGIC {
            return Err(Error::Invalid("refusing to write config block without 5A A5 marker".into()));
        }
        dev.write(&build_access(cmd::SETTINGS, 0, &self.0))
    }

    pub fn mode(&self) -> u8 {
        self.0[OFF_MODE]
    }
    pub fn set_mode(&mut self, mode: u8) {
        self.0[OFF_MODE] = mode;
        self.0[OFF_CUSTOM] = 0;
    }

    fn pair(&self, mode: u8) -> usize {
        OFF_PER_MODE + 2 * mode as usize
    }
    pub fn brightness(&self, mode: u8) -> u8 {
        self.0[self.pair(mode)]
    }
    pub fn set_brightness(&mut self, mode: u8, v: u8) {
        let i = self.pair(mode);
        self.0[i] = v.min(MAX_BRIGHTNESS);
    }
    pub fn speed(&self, mode: u8) -> u8 {
        self.0[self.pair(mode) + 1] >> 4
    }
    pub fn set_speed(&mut self, mode: u8, v: u8) {
        let i = self.pair(mode) + 1;
        self.0[i] = (v.min(MAX_SPEED) << 4) | (self.0[i] & 0x0F);
    }
    /// Palette index 0..=6, or [`COLOR_MIX`].
    pub fn color(&self, mode: u8) -> u8 {
        self.0[self.pair(mode) + 1] & 0x0F
    }
    pub fn set_color(&mut self, mode: u8, idx: u8) {
        let i = self.pair(mode) + 1;
        self.0[i] = (self.0[i] & 0xF0) | (idx & 0x0F);
    }

    pub fn sleep_minutes(&self) -> u8 {
        self.0[OFF_SLEEP]
    }
    pub fn set_sleep_minutes(&mut self, m: u8) {
        self.0[OFF_SLEEP] = m;
    }
    pub fn debounce(&self) -> u8 {
        self.0[OFF_DEBOUNCE]
    }
}

/// Per-mode 7-colour palettes. The firmware cannot report them, so the host keeps a copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palettes(pub [[[u8; 3]; PALETTE_COLORS]; PALETTE_MODES]);

pub const DEFAULT_PALETTE: [[u8; 3]; PALETTE_COLORS] =
    [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0], [255, 0, 255], [0, 255, 255], [255, 255, 255]];

impl Default for Palettes {
    fn default() -> Palettes {
        let mut p = [DEFAULT_PALETTE; PALETTE_MODES];
        p[0] = [[0; 3]; PALETTE_COLORS];
        Palettes(p)
    }
}

impl Palettes {
    pub fn report(&self) -> [u8; crate::packet::REPORT_LEN] {
        let body: Vec<u8> = self.0.iter().flatten().flatten().copied().collect();
        build_access(cmd::LED_RGB_TAB, 0, &body)
    }
    pub fn write(&self, dev: &Device) -> Result<()> {
        dev.write(&self.report())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Config {
        let mut b = [0u8; CONFIG_LEN];
        b[..16].copy_from_slice(&[0, 3, 3, 4, 0, 0, 0x13, 4, 0x0B, 0, 0x0C, 0x20, 1, 0, 0, 0]);
        b[0x38] = 0xFF;
        b[0x39] = 0xFF;
        for m in 1..30 {
            b[0x38 + 2 * m] = 0x14;
            b[0x39 + 2 * m] = 0x17;
        }
        b[126..].copy_from_slice(&MAGIC);
        Config(b)
    }

    /// The official app's first write after connecting: Fixed on, red, speed 1, brightness 20.
    #[test]
    fn reproduces_captured_change() {
        let mut c = sample();
        c.set_mode(1);
        c.set_color(1, 0);
        assert_eq!(c.0[10], 0x01);
        assert_eq!(&c.0[0x3A..0x3C], &[0x14, 0x10]);
        c.set_color(1, COLOR_MIX);
        assert_eq!(c.0[0x3B], 0x17);
        c.set_brightness(1, 0);
        assert_eq!(c.0[0x3A], 0x00);
    }

    #[test]
    fn palette_report_layout() {
        let mut p = Palettes::default();
        p.0[1][0] = [10, 20, 30];
        let r = p.report();
        assert_eq!(&r[..8], &[0x09, 0x0A, 0, 0, 1, 0, 0xA4, 0x01]); // 420 bytes
        assert_eq!(&r[8 + 21..8 + 27], &[10, 20, 30, 0, 255, 0]);
    }
}
