//! The screen's clock (SetScreenParam, command 0x0B, 14 bytes).
//! `00 00 00 <year u16 LE> <month> <day> <hour> <min> <sec> <weekday 0=Sun> 00 00 00`

use crate::device::Device;
use crate::keymap::build_access;
use crate::packet::cmd;
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// 0 = Sunday.
    pub weekday: u8,
}

impl DateTime {
    pub fn body(&self) -> [u8; 14] {
        let [y0, y1] = self.year.to_le_bytes();
        [0, 0, 0, y0, y1, self.month, self.day, self.hour, self.minute, self.second, self.weekday, 0, 0, 0]
    }

    pub fn write(&self, dev: &Device) -> Result<()> {
        dev.write(&build_access(cmd::SCREEN_PARAM, 0, &self.body()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_capture() {
        // 2026-10-20 20:04:44, Tuesday
        let t = DateTime { year: 2026, month: 10, day: 20, hour: 20, minute: 4, second: 44, weekday: 2 };
        assert_eq!(t.body(), [0, 0, 0, 0xEA, 0x07, 0x0A, 0x14, 0x14, 0x04, 0x2C, 0x02, 0, 0, 0]);
    }
}
