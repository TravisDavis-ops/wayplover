#![allow(dead_code)]
/*
 * What is a worker
 *  a worker is a collection of threads that produces events
 */
use crate::{steno::Stroke, BYTES_PER_STROKE, STENO_MAP};
use rodio::{
    source::{SineWave, Source},
    OutputStream, Sink,
};
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
                .spawn(move || loop {
                    let stdin = io::stdin();
                    for key in stdin.keys() {
                        if let Ok(key) = key {
                            if let Err(err) = tx.send(InputEvents::Window(key)) {
                                return;
                            }
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
                        break;
                    }
                    thread::sleep(config.tick_rate);
                })
        };
        Self {
            rx,
            h_window: h_window.expect("Failed start."),
            h_tick: h_tick.expect("Failed start."),
        }
    }
    pub fn poll(&self) -> Result<InputEvents, mpsc::RecvError> {
        self.rx.recv()
    }
}
pub enum Sound {
    Error,
    Success,
}

pub enum DeviceStatus {
    Input(Stroke),
}
pub enum DeviceControl {
    Disable,
    Enable,
    Reconnect(&'static str),
    Disconnect,
}
pub struct DeviceWorker {
    tx: mpsc::Sender<DeviceControl>,
    rx: mpsc::Receiver<DeviceStatus>,
    handler: thread::JoinHandle<()>,
}
impl Worker<DeviceControl, DeviceStatus> for DeviceWorker {
    fn start(config: Config) -> Self {
        let ((tx, thread_rx), (thread_tx, rx)) = (mpsc::channel(), mpsc::channel());
        let handler = thread::Builder::new()
            .name("DeviceThread".to_string())
            .spawn(move || {
                let mut port = serial::open(&config.port).expect("Decice Not Found");
                loop {
                    // was a message send
                    if let Ok(e) = thread_rx.try_recv() {
                        use DeviceControl::*;
                        match e {
                            Disconnect => {
                                break;
                            }
                            Reconnect(new_path) => {
                                port = serial::open(new_path).unwrap();
                                continue;
                            }
                            _ => {}
                        }
                    }
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
                        thread_tx.send(DeviceStatus::Input(stroke)).unwrap();
                    }
                }
            })
            .unwrap();
        Self { tx, rx, handler }
    }

    fn send(&self, e: DeviceControl) {
        todo!()
    }

    fn recv(&self) -> Option<DeviceStatus> {
        if let Ok(status) = self.rx.try_recv() {
            Some(status)
        } else {
            None
        }
    }
}
use tts::Tts;
pub enum AudioStatus {
    Volume(f32),
}

pub enum AudioControl {
    Play(Sound),
    Speak(String),
    Volume(Option<f32>),
    Stop,
}
pub struct AudioWorker {
    tx: mpsc::Sender<AudioControl>,
    rx: mpsc::Receiver<AudioStatus>,
    handler: thread::JoinHandle<()>,
}

impl Worker<AudioControl, AudioStatus> for AudioWorker {
    fn start(_c: Config) -> Self {
        let (thread_tx, thread_rx) = mpsc::channel();
        let (worker_tx, worker_rx) = mpsc::channel();
        let handler = thread::Builder::new()
            .name("AudioThread".to_string())
            .spawn(move || {
                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                let sink = Sink::try_new(&stream_handle).unwrap();
                let mut engine = Tts::default().unwrap();
                loop {
                    if let Ok(e) = thread_rx.try_recv() {
                        match e {
                            AudioControl::Play(Sound::Error) => {
                                let sound =
                                    SineWave::new(250).take_duration(Duration::from_millis(5));
                                sink.append(sound);
                            }
                            AudioControl::Play(Sound::Success) => {
                                let sound =
                                    SineWave::new(440).take_duration(Duration::from_millis(10));
                                sink.append(sound);
                            }

                            AudioControl::Volume(None) => {
                                worker_tx.send(AudioStatus::Volume(sink.volume())).unwrap();
                            }
                            AudioControl::Speak(word) => {
                                engine.speak(word, true).unwrap();
                            }
                            _ => {
                                sink.stop();
                            }
                        }
                    }
                    sink.sleep_until_end();
                }
            })
            .unwrap();
        Self {
            tx: thread_tx,
            rx: worker_rx,
            handler,
        }
    }

    fn send(&self, e: AudioControl) {
        self.tx.send(e).unwrap();
    }

    fn recv(&self) -> Option<AudioStatus> {
        todo!()
    }
}

pub trait Worker<In, Out> {
    fn start(c: Config) -> Self;
    fn send(&self, e: In);
    fn recv(&self) -> Option<Out>;
}
