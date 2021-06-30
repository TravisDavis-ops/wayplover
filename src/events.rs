#![allow(unused_imports)]
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::steno::Stroke;
use crate::*;
use termion::event::Key;
use termion::input::TermRead;

pub enum Event {
    Input(Key),
    Steno(Stroke),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event>,
    input_handle: thread::JoinHandle<()>,
    steno_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                    }
                }
            })
        };
        let steno_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                // lazy-static this
                let mut port = serial::open("/dev/ttyACM1").unwrap();
                loop {
                    let mut steno_buffer: [u8; BYTES_PER_STROKE] = [0, 0, 0, 0, 0, 0];
                    port.read_exact(&mut steno_buffer);
                    let mut steno_out = Vec::new();
                    for (i, b) in steno_buffer.iter().enumerate() {
                        for j in 1..8 {
                            if b & (0x80 >> j) != 0 {
                                steno_out.push(STENO_MAP[i * 7 + j - 1]);
                            }
                        }
                    }
                    let stroke = Stroke::new(steno_out);
                    if !stroke.is_empty(){
                        tx.send(Event::Steno(stroke));
                    }
                }

            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                if let Err(err) = tx.send(Event::Tick) {
                    eprintln!("{}", err);
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            input_handle,
            steno_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}
