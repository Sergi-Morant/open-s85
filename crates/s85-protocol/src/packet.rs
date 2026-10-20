//! Building and parsing the fixed-size reports.

pub const REPORT_ID: u8 = 0x09;
/// Feature report length including the report ID byte.
pub const REPORT_LEN: usize = 520;
pub const HEADER_LEN: usize = 8;
pub const MAX_BODY: usize = REPORT_LEN - HEADER_LEN;

/// Command bytes (second byte of every report). Setting bit 7 turns a write into a read.
pub mod cmd {
    pub const SET_MATRIX: u8 = 0x03;
    pub const SETTINGS: u8 = 0x04;
    pub const SET_GAME: u8 = 0x06;
    pub const REAL_DATA: u8 = 0x08;
    pub const LED_RGB_TAB: u8 = 0x0A;
    pub const SCREEN_PARAM: u8 = 0x0B;
    pub const SCREEN_DATA: u8 = 0x0C;
    pub const SCREEN_CONTROL: u8 = 0x0D;
    pub const RESET: u8 = 0x11;
    pub const READ: u8 = 0x80;
}

/// One host->keyboard report.
pub fn build(cmd: u8, frame: u8, idx: u16, body: &[u8]) -> [u8; REPORT_LEN] {
    assert!(body.len() <= MAX_BODY, "body too long: {}", body.len());
    let mut p = [0u8; REPORT_LEN];
    p[0] = REPORT_ID;
    p[1] = cmd;
    p[3] = frame;
    p[4..6].copy_from_slice(&idx.to_le_bytes());
    p[6..8].copy_from_slice(&(body.len() as u16).to_le_bytes());
    p[HEADER_LEN..HEADER_LEN + body.len()].copy_from_slice(body);
    p
}

/// An 8-byte input report from the keyboard: `09 0A 06 <status> <idx> 00 00 00`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ack {
    pub status: u8,
    pub idx: u8,
}

impl Ack {
    /// Parse an input report; some hidapi backends strip the report ID, so both forms are accepted.
    pub fn parse(r: &[u8]) -> Option<Ack> {
        let r = if r.first() == Some(&REPORT_ID) { &r[1..] } else { r };
        match r {
            [0x0A, 0x06, status, idx, ..] => Some(Ack { status: *status, idx: *idx }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_layout() {
        let p = build(cmd::SCREEN_DATA, 1, 0xD6, &[0xAB; 512]);
        assert_eq!(&p[..8], &[0x09, 0x0C, 0x00, 0x01, 0xD6, 0x00, 0x00, 0x02]);
        assert_eq!(p[8], 0xAB);
    }

    #[test]
    fn ack_with_and_without_report_id() {
        let a = Ack { status: 1, idx: 0x73 };
        assert_eq!(Ack::parse(&[0x09, 0x0A, 0x06, 0x01, 0x73, 0, 0, 0]), Some(a));
        assert_eq!(Ack::parse(&[0x0A, 0x06, 0x01, 0x73, 0, 0, 0]), Some(a));
        assert_eq!(Ack::parse(&[0x09, 0x0B, 0, 0]), None);
    }
}
