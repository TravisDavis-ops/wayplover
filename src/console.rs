#![allow(unused_imports)]
use std::io::stdout;
use std::io::Write;
use termion::*;
use termion::input::*;
use termion::event::Key;
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Console{
}
impl Console {
    pub fn default() -> Result<Self, std::io::Error> {
        Ok(Self { })
    }
    pub fn flush() -> Result<(), std::io::Error> {
        stdout().flush()
    }
    pub fn clear_screen() {
        print!("{}", clear::All);
    }
    pub fn clear_line() {
        print!("{}", clear::CurrentLine);
    }
    pub fn position(x: u16, y: u16) {
        print!("{}", cursor::Goto(x + 1, y + 1))
    }
    pub fn cursor_hide() {
        print!("{}", cursor::Hide);
    }
    pub fn cursor_show() {
        print!("{}", cursor::Show);
    }
    pub fn read_keys () -> Vec<Key> {
        let mut key_frame = Vec::new();
        for key in async_stdin().keys(){
            key_frame.push(key.unwrap());
        }
        key_frame
    }
}
impl Default for Console {
    fn default() -> Self {
        Self {
        }
    }
}
