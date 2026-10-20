//! Finding and talking to the keyboard over hidapi.

use std::time::{Duration, Instant};

use hidapi::{HidApi, HidDevice};

use crate::packet::{self, Ack, REPORT_LEN};
use crate::{Error, Result, PIDS, VID};

const USAGE_PAGE: u16 = 0xFF02;
const USAGE: u16 = 0x02;
const INTERFACE: i32 = 1;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub pid: u16,
    pub name: &'static str,
}

pub struct Device {
    hid: HidDevice,
    pub info: DeviceInfo,
    /// How long to wait for each acknowledgement.
    pub ack_timeout: Duration,
    /// How many times to resend an unacknowledged report.
    pub retries: u32,
}

impl Device {
    /// Open the first connected RK S85.
    pub fn open() -> Result<Device> {
        let api = HidApi::new()?;
        let known = |pid: u16| PIDS.iter().find(|(p, _)| *p == pid).map(|(_, n)| *n);
        let candidates: Vec<_> = api
            .device_list()
            .filter(|d| d.vendor_id() == VID && known(d.product_id()).is_some())
            .collect();
        // Windows and macOS expose one entry per top-level collection with its usage; Linux hidraw
        // exposes one node per interface, where usage may be unavailable.
        let pick = candidates
            .iter()
            .find(|d| d.usage_page() == USAGE_PAGE && d.usage() == USAGE)
            .or_else(|| candidates.iter().find(|d| d.interface_number() == INTERFACE))
            .ok_or(Error::NotFound)?;
        let info = DeviceInfo { pid: pick.product_id(), name: known(pick.product_id()).unwrap() };
        let hid = pick.open_device(&api)?;
        Ok(Device { hid, info, ack_timeout: Duration::from_millis(1000), retries: 5 })
    }

    /// Send one report and wait for an acknowledgement matching `expect_idx` (any if `None`).
    pub fn send(&self, report: &[u8; REPORT_LEN], expect_idx: Option<u8>) -> Result<Ack> {
        let mut last_err = None;
        for _ in 0..self.retries {
            // The keyboard can be briefly busy (e.g. another app polling it); a timed-out
            // transfer is retried like a missing acknowledgement.
            if let Err(e) = self.hid.send_feature_report(report) {
                last_err = Some(e);
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            if let Some(ack) = self.wait_ack(expect_idx)? {
                return Ok(ack);
            }
        }
        match last_err {
            Some(e) => Err(e.into()),
            None => Err(Error::NoAck(format!("command {:#04x} idx {:?}", report[1], expect_idx))),
        }
    }

    /// Send a report that the keyboard does not acknowledge (the official app's `AccessData` writes),
    /// retrying transfer errors like the official app does (3 tries, 50 ms apart).
    pub fn write(&self, report: &[u8; REPORT_LEN]) -> Result<()> {
        let mut last = None;
        for _ in 0..3 {
            match self.hid.send_feature_report(report) {
                Ok(()) => {
                    std::thread::sleep(Duration::from_millis(25));
                    return Ok(());
                }
                Err(e) => last = Some(e),
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Err(last.unwrap().into())
    }

    /// `AccessData` read: send `cmd` (with bit 7 set) and return the `len`-byte payload of the reply.
    pub fn read_access(&self, read_cmd: u8, b2: u8, len: usize) -> Result<Vec<u8>> {
        let mut req = packet::build(read_cmd, 0, 0, &[]);
        req[2] = b2;
        req[4] = 1;
        req[6..8].copy_from_slice(&(len as u16).to_le_bytes());
        self.write(&req)?;
        let mut buf = vec![0u8; REPORT_LEN];
        buf[0] = packet::REPORT_ID;
        let n = self.hid.get_feature_report(&mut buf)?;
        // Some backends drop the report ID from the returned buffer.
        let reply = if buf[0] == packet::REPORT_ID { &buf[..n] } else { &buf[..n.min(REPORT_LEN)] };
        let off = if reply.first() == Some(&packet::REPORT_ID) { 0 } else { 1 };
        let hdr = |i: usize| reply.get(i - off).copied().unwrap_or(0);
        if hdr(1) != read_cmd {
            return Err(Error::Invalid(format!("reply to {read_cmd:#04x} has command {:#04x}", hdr(1))));
        }
        let start = packet::HEADER_LEN - off;
        if reply.len() < start + len {
            return Err(Error::Invalid(format!("short reply: {} bytes", reply.len())));
        }
        Ok(reply[start..start + len].to_vec())
    }

    /// Discard stale input reports (e.g. replies to another app's polling).
    pub fn drain(&self) -> Result<()> {
        let mut buf = [0u8; 64];
        while self.hid.read_timeout(&mut buf, 10)? > 0 {}
        Ok(())
    }

    fn wait_ack(&self, expect_idx: Option<u8>) -> Result<Option<Ack>> {
        let deadline = Instant::now() + self.ack_timeout;
        let mut buf = [0u8; 64];
        while let Some(left) = deadline.checked_duration_since(Instant::now()) {
            let n = self.hid.read_timeout(&mut buf, left.as_millis().max(1) as i32)?;
            if let Some(ack) = Ack::parse(&buf[..n]) {
                if ack.status == 0x01 && expect_idx.is_none_or(|i| i == ack.idx) {
                    return Ok(Some(ack));
                }
            }
        }
        Ok(None)
    }
}
