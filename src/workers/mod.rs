use std::time::Duration;
pub mod serial;
#[cfg(feature = "sound")]
pub mod sound;
pub mod window;

pub trait Worker<In, Out> {
    fn start(c: Config) -> Self;
    fn send(&self, e: In);
    fn recv(&self) -> Option<Out>;
    fn shutdown(&self);
}
pub trait Shutdown<In, Out> {
    fn shutdown<T: Worker<In, Out>>(worker: &T) {
        worker.shutdown();
    }
}
pub struct WorkerPool {
    #[cfg(feature = "sound")]
    pub audio: sound::AudioWorker,
    pub serial: serial::SerialWorker,
    pub window: window::InputWorker,
}

impl Shutdown<window::InputControl, window::InputStatus> for WorkerPool {}
impl Shutdown<serial::DeviceControl, serial::DeviceStatus> for WorkerPool {}
#[cfg(feature = "sound")]
impl Shutdown<sound::AudioControl, sound::AudioStatus> for WorkerPool {}

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
