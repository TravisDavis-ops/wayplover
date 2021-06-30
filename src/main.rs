#![allow(unused_imports)]
#[macro_export]
use lazy_static::lazy_static;
use maplit::hashmap;
use serial;
use serial::unix::TTYPort;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::stdout;
use std::ops::Add;
use std::thread::sleep;
use std::time::Duration;
use termion::*;
pub mod console;
pub mod editor;
pub mod events;
pub mod steno;

use editor::Editor;

use json;
type Position = (u16, u16);

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
    let mut app = Editor::default();
    app.run();
}
