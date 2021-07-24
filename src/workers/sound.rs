use crate::workers::Worker;
use crate::{steno::Stroke, BYTES_PER_STROKE, STENO_MAP};
use rodio::{
    source::{SineWave, Source},
    OutputStream, Sink,
};
use std::io::{self, Read};
use std::sync::mpsc;
use std::thread;
use std::time::*;
use termion::{event::Key, input::TermRead};
use tts::Tts;
pub enum AudioStatus {
    Volume(f32),
}
pub enum Sound {
    Error,
    Success,
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
