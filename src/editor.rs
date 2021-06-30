use crate::console::Console;
use crate::events;
use crate::steno::*;
use crate::*;
use evdev::uinput;
use evdev::{AttributeSet, Device, Key};
use serial;
use serial::unix::TTYPort;
use std::thread::*;
use std::time::Duration;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::widgets::*;
use tui::Terminal;
type Position = (u16, u16);

pub struct Editor {
    terminal: Terminal<TermionBackend<RawTerminal<std::io::Stdout>>>,
    device: uinput::VirtualDevice,
    dict: Dictonary,
    history: Events,
}
impl Default for Editor {
    fn default() -> Self {
        let dict = Dictonary::from_file("./main.json");
        let backend = TermionBackend::new(stdout().into_raw_mode().unwrap());
        let terminal = tui::Terminal::new(backend).unwrap();
        let history = Events::new(Vec::new(), terminal.size().unwrap().height as usize);
        let keys = key_set!(
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
            Key::KEY_BACKSPACE,
            Key::KEY_SPACE,
            Key::KEY_LEFTCTRL
        );
        let device = uinput::VirtualDeviceBuilder::new()
            .unwrap()
            .name("wayplover")
            .with_keys(&keys)
            .unwrap()
            .build()
            .unwrap();
        Console::clear_screen();
        Self {
            terminal,
            device,
            history,
            dict,
        }
    }
}
#[derive(Clone)]
struct Events {
    // `items` is the state managed by your application.
    items: Vec<String>,
    // `state` is the state that can be modified by the UI. It stores the index of the selected
    // item as well as the offset computed during the previous draw call (used to implement
    // natural scrolling).
    max_size: usize,
    state: ListState,
}
impl Events {
    fn new(items: Vec<String>, max_size: usize) -> Events {
        Events {
            items,
            max_size,
            state: ListState::default(),
        }
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        if self.items.len() > self.max_size {
            self.items.clear();
            self.unselect();
        }

        self.items.extend(items);
    }

    pub fn select_new(&mut self) {
        self.state.select(Some(0));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

impl Editor {
    fn handle_stroke(&mut self, stroke: Stroke) {
        use evdev::*;
        let mut seq = Vec::new();
        for (i, key) in stroke.resolve(&mut self.dict).split("").enumerate() {
            if key == "*" && i == 1 {
                seq.push(Key::KEY_LEFTCTRL);
                seq.push(Key::KEY_BACKSPACE);
                break;
            } else {
                match key {
                    "a" | "A" => seq.push(Key::new(30)),
                    "b" | "B" => seq.push(Key::new(48)),
                    "c" | "C" => seq.push(Key::new(46)),
                    "d" | "D" => seq.push(Key::new(32)),
                    "e" | "E" => seq.push(Key::new(18)),
                    "f" | "F" => seq.push(Key::new(33)),
                    "g" | "G" => seq.push(Key::new(34)),
                    "h" | "H" => seq.push(Key::new(35)),
                    "i" | "I" => seq.push(Key::new(23)),
                    "j" | "J" => seq.push(Key::new(36)),
                    "k" | "K" => seq.push(Key::new(37)),
                    "l" | "L" => seq.push(Key::new(38)),
                    "m" | "M" => seq.push(Key::new(50)),
                    "n" | "N" => seq.push(Key::new(49)),
                    "o" | "O" => seq.push(Key::new(24)),
                    "p" | "P" => seq.push(Key::new(25)),
                    "q" | "Q" => seq.push(Key::new(16)),
                    "r" | "R" => seq.push(Key::new(19)),
                    "s" | "S" => seq.push(Key::new(31)),
                    "t" | "T" => seq.push(Key::new(20)),
                    "u" | "U" => seq.push(Key::new(22)),
                    "v" | "V" => seq.push(Key::new(47)),
                    "w" | "W" => seq.push(Key::new(17)),
                    "x" | "X" => seq.push(Key::new(45)),
                    "y" | "Y" => seq.push(Key::new(21)),
                    "z" | "Z" => seq.push(Key::new(44)),
                    _ => {}
                }
            }
        }
        let mut ctrl = false;
        let len = seq.len();
        for (i, &key) in seq.iter().enumerate() {
            let down = InputEvent::new(EventType::KEY, key.code(), 1);
            let up = InputEvent::new(EventType::KEY, key.code(), 0);
            match key {
                Key::KEY_LEFTCTRL | Key::KEY_LEFTALT => {
                    self.device.emit(&[down]);
                    sleep(Duration::from_millis(2));
                    ctrl = true;
                }
                _ => {
                    self.device.emit(&[down]);
                    sleep(Duration::from_millis(2));
                    self.device.emit(&[up]);
                    sleep(Duration::from_millis(2));
                    if ctrl {
                        self.device.emit(&[InputEvent::new(
                            EventType::KEY,
                            Key::KEY_LEFTCTRL.code(),
                            0,
                        )]);
                        self.history.unselect();
                        sleep(Duration::from_millis(2));
                    } else if i.eq(&(len - 1)) {
                        let down = InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 1);
                        let up = InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 0);
                        self.device.emit(&[down]);
                        sleep(Duration::from_millis(2));
                        self.device.emit(&[up]);
                        sleep(Duration::from_millis(2));
                    }
                }
            }
        }
        self.history.set_items(vec![stroke.resolve(&mut self.dict)]);
        self.history.select_new();
    }
    pub fn run(&mut self) {
        use tui::widgets::*;
        let event = events::Events::new();
        loop {
            Console::position(0, 0);
            match event.next().unwrap() {
                events::Event::Steno(s) => {
                    self.handle_stroke(s);
                }
                events::Event::Input(termion::event::Key::Ctrl(key)) => {
                    if key.eq(&'c'){
                        Console::clear_screen();
                        return;
                    }
                }
                events::Event::Tick => {}
                _=>{}
            }
            let mut history = self.history.clone();
            self.terminal
                .draw(|f| {
                    use tui::style::*;
                    let size = f.size();
                    let max = history.max_size.clone();
                    let items: Vec<ListItem> = history
                        .items
                        .iter()
                        .map(|i| ListItem::new(i.as_ref()))
                        .collect();
                    let block = Block::default()
                        .title(format!("Paper Tape: {}/{}", items.len(), max))
                        .borders(Borders::all());
                    let list = List::new(items.into_iter().rev().collect::<Vec<ListItem>>())
                        .block(block)
                        .highlight_symbol(">>");
                    f.render_stateful_widget(list, size, &mut history.state);
                })
                .unwrap();
            Console::position(1, 1);
            self.terminal.autoresize().unwrap();
        }
    }

    fn valdate_stroke(&self, stroke_buffer: [u8; BYTES_PER_STROKE]) -> bool {
        // todo( health ) no magic
        let magic = stroke_buffer[0] & 0x80;
        let magic_list = stroke_buffer
            .iter()
            .map(|elem| elem & 0x80)
            .filter(|e| *e != 0x00)
            .collect::<Vec<u8>>();

        magic == stroke_buffer[0] && magic_list.len() == 1
    }
}
use std::fs;
pub struct Document {
    pub rows: Vec<Row>,
}
impl Document {
    pub fn open(filename: &str) -> Self {
        let mut rows = Vec::new();
        let file = fs::read_to_string(filename).unwrap();
        for line in file.lines() {
            rows.push(Row::from(line));
        }
        Self { rows }
    }
    pub fn insert_row(&mut self, s: &str) {
        if !s.is_empty() {
            self.rows.push(Row::from(s));
        }
    }
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

use std::cmp;
pub struct Row {
    _body: String,
}
impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self._body.len());
        let start = cmp::min(start, end);
        self._body.get(start..end).unwrap_or_default().to_string()
    }
}
impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            _body: String::from(slice),
        }
    }
}
