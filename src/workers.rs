#![allow(dead_code)]
/*
 * What is a worker
 *  a worker is a collection of threads that produces events
 */
use crate::{steno::Stroke, BYTES_PER_STROKE, STENO_MAP};
use serial;
use std::io::{self, Read};
use std::sync::mpsc;
use std::thread;
use std::time::*;
use termion::{event::Key, input::TermRead};
#[derive(Clone)]
pub struct Config {
    pub tick_rate: Duration,
    pub port: String,
}
impl Default for Config {
    fn default() -> Config {
        Self {
            tick_rate: Duration::from_millis(250),
            port: "/dev/ttyACM0".to_string(),
        }
    }
}
#[derive(Debug)]
pub enum InputEvents {
    Window(Key),
    Device(Stroke),
    Tick,
}
pub struct InputWorker {
    rx: mpsc::Receiver<InputEvents>,
    h_window: thread::JoinHandle<()>,
    h_device: thread::JoinHandle<()>,
    h_tick: thread::JoinHandle<()>,
}
impl Default for InputWorker {
    fn default() -> Self {
        Self::new()
    }
}
impl InputWorker {
    fn new() -> Self {
        Self::with_config(Config::default())
    }
    pub fn with_config(config: Config) -> Self {
        use std::thread::Builder;
        let (tx, rx) = mpsc::channel();
        let h_window = {
            let tx = tx.clone();
            let _config = config.clone();
            Builder::new()
                .name("WindowInput".to_string())
                .spawn(move || {
                    let stdin = io::stdin();
                    for key in stdin.keys() {
                        if let Ok(key) = key {
                            if let Err(err) = tx.send(InputEvents::Window(key)) {
                                eprintln!("{}", err);
                                return;
                            }
                        }
                    }
                })
        };
        let h_device = {
            let tx = tx.clone();
            let config = config.clone();
            Builder::new()
                .name("DeviceInput".to_string())
                .spawn(move || {
                    let mut port = serial::open(&config.port).unwrap();
                    loop {
                        let mut b_stroke: [u8; BYTES_PER_STROKE] = [0; BYTES_PER_STROKE];
                        if let Err(_e) = port.read_exact(&mut b_stroke) {
                            continue;
                        };

                        let mut o_steno = Vec::new();
                        for (i, b) in b_stroke.iter().enumerate() {
                            for j in 1..8 {
                                if b & (0x80 >> j) != 0 {
                                    o_steno.push(STENO_MAP[i * 7 + j - 1]);
                                }
                            }
                        }
                        let stroke = Stroke::new(o_steno);
                        if !stroke.is_empty() {
                            tx.send(InputEvents::Device(stroke)).unwrap();
                        }
                    }
                })
        };
        let h_tick = {
            let tx = tx.clone();
            let config = config.clone();
            Builder::new()
                .name("WorkerHeart".to_string())
                .spawn(move || loop {
                    if let Err(err) = tx.send(InputEvents::Tick) {
                        eprintln!("{}", err);
                        break;
                    }
                    thread::sleep(config.tick_rate);
                })
        };
        Self {
            rx,
            h_window: h_window.expect("Failed start."),
            h_device: h_device.expect("Failed start."),
            h_tick: h_tick.expect("Failed start."),
        }
    }
    pub fn poll(&self) -> Result<InputEvents, mpsc::RecvError> {
        self.rx.recv()
    }
}
enum AudioEvent {

}
struct AudioWorker {
   tx: mpsc::Receiver<AudioEvent>,
   audio: thread::JoinHandle<()>,
}

