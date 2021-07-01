use lazy_static::lazy_static;
use clap::{Arg, App};
use std::collections::{HashMap, HashSet};
use maplit::hashmap;
use std::time::Duration;
use json;
use std::fs::File;
use std::io::prelude::*;
use std::thread::sleep;
pub mod ui;
pub mod steno;
pub mod worker;
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
    static ref STENO_ORDER: HashMap<&'static str, i8> = {
        hashmap! {
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
fn main() {

    let app = App::new(NAME)
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .value_name("PORT")
             .help("The device name ie /dev/ttyACM0."))
        .arg(Arg::with_name("dictonary")
              .short("d")
              .long("dictonary")
              .value_name("DICTONARY")
              .required(true)
              .help("The dictonary file to use."));
    let matches = app.get_matches();
    let port = matches.value_of("port").unwrap_or("/dev/ttyACM0");
    let path = matches.value_of("dictonary").unwrap();
    let config  = worker::Config{ tick_rate: Duration::from_secs(5), port: port.to_string()};
    let worker = worker::InputWorker::with_config(config);
    let dictonary = steno::Dictonary::from_file(path);

    let mut ui = ui::Ui::new(worker, dictonary);
    ui.run();
    //todo rename Ui
    //let mut app = Editor::default();
    //app.run();
}
