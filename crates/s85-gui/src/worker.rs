//! Background thread that owns the keyboard connection, so the UI never blocks on USB.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use s85_protocol::clock::DateTime;
use s85_protocol::keymap::{Keymap, Layer};
use s85_protocol::lighting::{Config, Palettes};
use s85_protocol::screen::Animation;
use s85_protocol::Device;

pub enum Job {
    /// Open the keyboard (if needed), sync the clock and read the lighting config.
    Connect,
    WriteConfig(Config),
    WritePalettes(Palettes),
    WriteLayer(Box<Keymap>, Layer),
    /// Write every layer and the palettes (used when switching profiles).
    WriteAll(Box<Keymap>, Palettes),
    SyncClock,
    Upload(Box<Animation>),
}

pub enum Event {
    Connected(String),
    Disconnected(String),
    Config(Config),
    Progress(usize, usize),
    Done(String),
    Error(String),
}

pub struct Worker {
    tx: Sender<Job>,
    pub rx: Receiver<Event>,
}

impl Worker {
    pub fn spawn(ctx: eframe::egui::Context) -> Worker {
        let (tx, job_rx) = channel::<Job>();
        let (ev_tx, rx) = channel::<Event>();
        thread::spawn(move || run(job_rx, ev_tx, ctx));
        Worker { tx, rx }
    }

    pub fn send(&self, job: Job) {
        let _ = self.tx.send(job);
    }
}

fn now() -> DateTime {
    use chrono::{Datelike, Local, Timelike};
    let n = Local::now();
    DateTime {
        year: n.year() as u16,
        month: n.month() as u8,
        day: n.day() as u8,
        hour: n.hour() as u8,
        minute: n.minute() as u8,
        second: n.second() as u8,
        weekday: n.weekday().num_days_from_sunday() as u8,
    }
}

fn run(jobs: Receiver<Job>, events: Sender<Event>, ctx: eframe::egui::Context) {
    let mut dev: Option<Device> = None;
    let emit = |e: Event| {
        let _ = events.send(e);
        ctx.request_repaint();
    };
    for job in jobs {
        if dev.is_none() {
            match Device::open() {
                Ok(d) => {
                    emit(Event::Connected(d.info.name.to_string()));
                    dev = Some(d);
                }
                Err(e) => {
                    emit(Event::Disconnected(e.to_string()));
                    continue;
                }
            }
        }
        let d = dev.as_ref().unwrap();
        let result = match job {
            Job::Connect => now()
                .write(d)
                .and_then(|_| Config::read(d))
                .map(|c| emit(Event::Config(c))),
            Job::WriteConfig(c) => c.write(d),
            Job::WritePalettes(p) => p.write(d),
            Job::WriteLayer(km, layer) => km.write_layer(d, layer),
            // FN2/Tap factory contents are unknown, so they are only written once the user has
            // assigned something there.
            Job::WriteAll(km, pal) => Layer::ALL
                .iter()
                .filter(|&&l| matches!(l, Layer::Default | Layer::Fn1) || !km.layer_is_empty(l))
                .try_for_each(|&l| km.write_layer(d, l))
                .and_then(|_| pal.write(d))
                .map(|_| emit(Event::Done("Profile written to keyboard".into()))),
            Job::SyncClock => now().write(d).map(|_| emit(Event::Done("Clock synchronised".into()))),
            Job::Upload(anim) => anim
                .upload(d, |done, total| emit(Event::Progress(done, total)))
                .map(|_| emit(Event::Done("Image uploaded".into()))),
        };
        if let Err(e) = result {
            // Drop the handle so the next job reconnects (the keyboard may have been unplugged).
            dev = None;
            emit(Event::Error(e.to_string()));
        }
    }
}
