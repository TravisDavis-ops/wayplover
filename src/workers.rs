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

pub struct WorkerPool {
    pub audio: AudioWorker,
    pub device: DeviceWorker,
    pub input: InputWorker,
}
impl Shutdown<InputControl, InputStatus> for WorkerPool {}
impl Shutdown<DeviceControl, DeviceStatus> for WorkerPool {}
impl Shutdown<AudioControl, AudioStatus> for WorkerPool {}

pub trait Shutdown<In, Out> {
    fn shutdown<T: Worker<In, Out>>(worker: &T) {
        worker.shutdown();
    }
}
#[derive(Debug)]
pub enum InputStatus {
    Input(Key),
}
pub enum InputControl {
    Shutdown,
}

pub struct InputWorker {
    tx: mpsc::Sender<InputControl>,
    rx: mpsc::Receiver<InputStatus>,
    handler: thread::JoinHandle<()>,
}
impl InputWorker {
    /* does ui need to know what the read event is
     * */
}
impl Worker<InputControl, InputStatus> for InputWorker {
    fn start(config: Config) -> Self {
        use std::thread::Builder;
        let ((tx, thread_rx), (thread_tx, rx)) = (mpsc::channel(), mpsc::channel());
        let handler = {
            let _config = config.clone();
            Builder::new()
                .name("WindowInput".to_string())
                .spawn(move || loop {
                    let stdin = io::stdin();
                    if let Ok(e) = thread_rx.try_recv() {
                        use InputControl::*;
                        match e {
                            Shutdown => {
                                break;
                            }
                        }
                    }

                    for key in stdin.keys() {
                        if let Ok(key) = key {
                            if let Err(_err) = thread_tx.send(InputStatus::Input(key)) {
                                continue;
                            }
                        }
                    }
                })
                .unwrap()
        };
        Self { rx, tx, handler }
    }
    fn recv(&self) -> Option<InputStatus> {
        self.rx.try_recv().ok()
    }
    fn send(&self, e: InputControl) {
        self.tx.send(e).unwrap()
    }
    fn shutdown(&self) {
        self.send(InputControl::Shutdown);
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
    Shutdown,
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
                            Shutdown => {
                                break;
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
        self.tx.send(e).expect("Send Error");
    }
    fn shutdown(&self) {
        self.send(DeviceControl::Shutdown);
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
    Shutdown,
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
                    if let Ok(e) = thread_rx.recv() {
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
                            AudioControl::Shutdown => {
                                break;
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
    fn shutdown(&self) {
        self.send(AudioControl::Shutdown);
    }
}

pub trait Worker<In, Out> {
    fn start(c: Config) -> Self;
    fn send(&self, e: In);
    fn recv(&self) -> Option<Out>;
    fn shutdown(&self);
}
