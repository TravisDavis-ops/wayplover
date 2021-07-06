use crate::{steno::*, worker::InputWorker, *};
use evdev::{uinput, AttributeSet, Key};
use std::io::stdout;
use std::time::Duration;
use termion::{
    event::Key as StdKey,
    raw::{IntoRawMode, RawTerminal},
};
use tui::{backend::TermionBackend, widgets::*, Terminal};

pub struct Ui {
    terminal: Terminal<TermionBackend<RawTerminal<std::io::Stdout>>>,
    keyboard: uinput::VirtualDevice,
    worker: InputWorker,
    dictionary: Dictionary,
    chord_history: History,
    stroke_history: History,
}

impl Default for Ui {
    fn default() -> Self {
        let dictionary = Dictionary::from_file("./main.json");
        let backend = TermionBackend::new(stdout().into_raw_mode().unwrap());
        let mut terminal = tui::Terminal::new(backend).unwrap();
        let stroke_history = History::new(Vec::new(), terminal.size().unwrap().height as usize);
        let chord_history = History::new(Vec::new(), terminal.size().unwrap().height as usize);
        let worker = InputWorker::default();
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
        let keyboard = uinput::VirtualDeviceBuilder::new()
            .unwrap()
            .name("wayplover")
            .with_keys(&keys)
            .unwrap()
            .build()
            .unwrap();
        terminal.clear().unwrap();
        Self {
            terminal,
            keyboard,
            stroke_history,
            chord_history,
            dictionary,
            worker,
        }
    }
}

impl Ui {
    pub fn new(worker: InputWorker, dictionary: Dictionary) -> Self {
        let backend = TermionBackend::new(stdout().into_raw_mode().unwrap());
        let mut terminal = tui::Terminal::new(backend).unwrap();
        let chord_history = History::new(Vec::new(), terminal.size().unwrap().height as usize);
        let stroke_history = History::new(Vec::new(), terminal.size().unwrap().height as usize);
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
        let keyboard = uinput::VirtualDeviceBuilder::new()
            .unwrap()
            .name("wayplover")
            .with_keys(&keys)
            .unwrap()
            .build()
            .unwrap();
        terminal.clear().unwrap();
        Self {
            terminal,
            keyboard,
            chord_history,
            stroke_history,
            dictionary,
            worker,
        }
    }
    fn handle_strokes(&mut self, stroke: Stroke) {
        use evdev::*;
        let mut seq = Vec::new();
        for (i, key) in stroke.resolve(&mut self.dictionary).split("").enumerate() {
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
                    self.keyboard.emit(&[down]).unwrap();
                    sleep(Duration::from_millis(2));
                    ctrl = true;
                }
                _ => {
                    self.keyboard.emit(&[down]).unwrap();
                    sleep(Duration::from_millis(2));
                    self.keyboard.emit(&[up]).unwrap();
                    sleep(Duration::from_millis(2));
                    if ctrl {
                        self.keyboard
                            .emit(&[InputEvent::new(EventType::KEY, Key::KEY_LEFTCTRL.code(), 0)])
                            .unwrap();
                        sleep(Duration::from_millis(2));
                    } else if i.eq(&(len - 1)) {
                        let down = InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 1);
                        let up = InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 0);
                        self.keyboard.emit(&[down]).unwrap();
                        sleep(Duration::from_millis(2));
                        self.keyboard.emit(&[up]).unwrap();
                        sleep(Duration::from_millis(2));
                    }
                }
            }
        }
        self.chord_history
            .set_items(vec![stroke.resolve(&mut self.dictionary)]);
        self.chord_history.select_new();
        self.stroke_history.set_items(vec![stroke.plain()]);
        self.stroke_history.select_new();
    }

    fn handle_keys(&mut self, _key: StdKey) {}

    pub fn run(&mut self) {
        use crate::worker::InputEvents::*;
        use tui::widgets::*;
        loop {
            self.terminal.get_frame().set_cursor(0, 0);
            match self.worker.poll().unwrap() {
                Device(s) => {
                    self.handle_strokes(s);
                }
                Window(termion::event::Key::Ctrl(key)) => {
                    if key.eq(&'c') {
                        self.terminal.clear().unwrap();
                        return;
                    }
                }
                Window(key) => {
                    self.handle_keys(key);
                }
                Tick => {}
            }
            let mut chord_history = self.chord_history.clone();
            let mut stroke_history = self.stroke_history.clone();
            self.terminal
                .draw(|f| {
                    use tui::layout::*;
                    let size = f.size();
                    let segments = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([
                            Constraint::Percentage(70),
                            Constraint::Ratio(1,15),
                            Constraint::Percentage(20),
                        ].as_ref())
                        .split(size);

                    {
                        /*
                         * Chord History
                         * */
                        let max = chord_history.max_size.clone();
                        let chords: Vec<ListItem> = chord_history
                            .items
                            .iter()
                            .map(|i| ListItem::new(i.as_ref()))
                            .collect();
                        let container = Block::default()
                            .title(format!("Chord History: {}/{}", chords.len(), max))
                            .borders(Borders::all());
                        let chord_list =
                            List::new(chords.into_iter().rev().collect::<Vec<ListItem>>())
                                .block(container)
                                .highlight_symbol(">>");

                        f.render_stateful_widget(chord_list, segments[0], &mut chord_history.state);
                    }
                    {
                        /*
                         * Stroke History
                         * */
                        let max = stroke_history.max_size.clone();
                        let strokes: Vec<ListItem> = stroke_history
                            .items
                            .iter()
                            .map(|i| ListItem::new(i.as_ref()))
                            .collect();
                        let container = Block::default()
                            .title(format!("Stroke History: {}/{}", strokes.len(), max))
                            .borders(Borders::all());
                        let stroke_list =
                            List::new(strokes.into_iter().rev().collect::<Vec<ListItem>>())
                                .block(container)
                                .highlight_symbol(">>");

                        f.render_stateful_widget(stroke_list, segments[2], &mut stroke_history.state);
                    }
                })
                .unwrap();
            self.terminal.get_frame().set_cursor(1, 1);
            self.terminal.autoresize().unwrap();
        }
    }

}
#[derive(Clone)]
struct History {
    items: Vec<String>,
    max_size: usize,
    state: ListState,
}

impl History {
    fn new(items: Vec<String>, max_size: usize) -> Self {
        Self {
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
