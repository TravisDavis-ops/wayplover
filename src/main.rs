use clap::{App, Arg};
use json;
use lazy_static::lazy_static;
use maplit::hashmap;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::thread::sleep;
use std::time::Duration;

pub mod steno;
pub mod ui;
pub mod utils;
pub mod workers;

use evdev::{AttributeSet, Key};

const NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
//todo move steno const to steno mod
const BYTES_PER_STROKE: usize = 6;
const STENO_MAP: [&str; 42] = [
    "Fn", "#", "#", "#", "#", "#", "#", "S-", "S-", "T-", "K-", "P-", "W-", "H-", "R-", "A-", "O-",
    "*", "*", "res", "res", "pwr", "*", "*", "-E", "-U", "-F", "-R", "-P", "-B", "-L", "-G", "-T",
    "-S", "-D", "#", "#", "#", "#", "#", "#", "-Z",
];
lazy_static! {
    static ref KEY_CODE: HashMap<&'static str, (Option<Key>, Key)> = {
        hashmap! {
            "a" => (None, Key::KEY_A), "A" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_A),
            "b" => (None, Key::KEY_B), "B" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_B),
            "c" => (None, Key::KEY_C), "C" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_C),
            "d" => (None, Key::KEY_D), "D" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_D),
            "e" => (None, Key::KEY_E), "E" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_E),
            "f" => (None, Key::KEY_F), "F" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_F),
            "g" => (None, Key::KEY_G), "G" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_G),
            "h" => (None, Key::KEY_H), "H" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_H),
            "i" => (None, Key::KEY_I), "I" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_I),
            "j" => (None, Key::KEY_J), "J" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_J),
            "k" => (None, Key::KEY_K), "K" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_K),
            "l" => (None, Key::KEY_L), "L" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_L),
            "m" => (None, Key::KEY_M), "M" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_M),
            "n" => (None, Key::KEY_N), "N" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_N),
            "o" => (None, Key::KEY_O), "O" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_O),
            "p" => (None, Key::KEY_P), "P" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_P),
            "q" => (None, Key::KEY_Q), "Q" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_Q),
            "r" => (None, Key::KEY_R), "R" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_R),
            "s" => (None, Key::KEY_S), "S" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_S),
            "t" => (None, Key::KEY_T), "T" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_T),
            "u" => (None, Key::KEY_U), "U" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_U),
            "v" => (None, Key::KEY_V), "V" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_V),
            "w" => (None, Key::KEY_W), "W" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_W),
            "x" => (None, Key::KEY_X), "X" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_X),
            "y" => (None, Key::KEY_Y), "Y" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_Y),
            "z" => (None, Key::KEY_Z), "Z" => (Some(Key::KEY_LEFTSHIFT), Key::KEY_Z),
            "'" => (None, Key::KEY_APOSTROPHE), "\""=> (Some(Key::KEY_LEFTSHIFT), Key::KEY_APOSTROPHE)
        }
    };
    static ref VIRT_KEY_MAP: AttributeSet<Key> = {
        key_set!(
            Key::KEY_A,
            Key::KEY_B,
            Key::KEY_C,
            Key::KEY_D,
            Key::KEY_E,
            Key::KEY_F,
            Key::KEY_G,
            Key::KEY_H,
            Key::KEY_I,
            Key::KEY_J,
            Key::KEY_K,
            Key::KEY_L,
            Key::KEY_M,
            Key::KEY_N,
            Key::KEY_O,
            Key::KEY_P,
            Key::KEY_Q,
            Key::KEY_R,
            Key::KEY_S,
            Key::KEY_T,
            Key::KEY_U,
            Key::KEY_V,
            Key::KEY_W,
            Key::KEY_X,
            Key::KEY_Y,
            Key::KEY_Z,
            Key::KEY_APOSTROPHE,
            Key::KEY_LEFTSHIFT,
            Key::KEY_LEFTALT,
            Key::KEY_LEFTCTRL,
            Key::KEY_LEFTMETA,
            Key::KEY_BACKSPACE,
            Key::KEY_SPACE
        )
    };
    static ref STENO_ORDER: utils::OrderedMap<&'static str, i8, i8> = {
        ordered_map! {
             "#"=> 0  ,  "S-"=> 1 ,  "T-"=> 2 ,  "K-"=> 3 ,  "P-"=> 4 ,
             "W-"=> 5 ,  "H-"=> 6 ,  "R-"=> 7 ,  "A-"=> 8 ,  "O-"=> 9 ,
             "*"=> 10 , "-E"=> 11 , "-U"=> 12 , "-F"=> 13 , "-R"=> 14 ,
            "-P"=> 15 , "-B"=> 16 , "-L"=> 17 , "-G"=> 18 , "-T"=> 19 ,
            "-S"=> 20 , "-D"=> 21 , "-Z"=> 22 ,
        }
    };
    static ref STENO_NUMBERS: HashMap<&'static str, &'static str> = {
        hashmap! {
            "S-"=>"1-",
            "T-"=> "2-",
            "P-"=> "3-",
            "H-"=> "4-",
            "A-"=> "5-",
            "O-"=> "0-",
            "-F"=> "-6",
            "-P"=> "-7",
            "-L"=> "-8",
            "-T"=> "-9",
        }
    };
}
#[macro_export]
macro_rules! key_set{
    ( $($n:expr), *) => {{
            let mut temp_key_set = AttributeSet::<Key>::new();
            $(
                temp_key_set.insert($n);
            )*
            temp_key_set
    }};
}
#[macro_export]
macro_rules! ordered_map {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr), *) => (<[()]>::len(&[$(ordered_map!(@single $rest)), *]));
    ($($key:expr => $value:expr, ) +) => {
        ordered_map!($($key => $value), +)
    };
    ($($key:expr => $value:expr), *) => {
        let _cap = ordered_map!(@count $($key), *);
        let mut _map = utils::OrderedMap::new(|&v|{v});
        $(
            let _ = _map.insert($key, $value);
        )*
        _map
    }


}
fn main() {
    let app = App::new(NAME)
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("The device name ie /dev/ttyACM0."),
        )
        .arg(
            Arg::with_name("dictionary")
                .short("d")
                .long("dictionary")
                .value_name("dictionary")
                .help("The dictionary file to use."),
        );
    let matches = app.get_matches();
    let port = matches.value_of("port").unwrap_or("/dev/ttyACM0");
    let path = matches.value_of("dictionary").unwrap_or("./main.json");
    let config = workers::Config {
        tick_rate: Duration::from_millis(5),
        port: port.to_string(),
    };
    use workers::Worker;
    let worker_pool = ui::WorkerPool {
        audio: workers::AudioWorker::start(config.clone()),
        device: workers::DeviceWorker::start(config.clone()),
        input: workers::InputWorker::with_config(config.clone()),
    };
    let dictionary = steno::Dictionary::from_file(path);

    let mut ui = ui::Tui::new(worker_pool, dictionary);
    ui.run();
}
