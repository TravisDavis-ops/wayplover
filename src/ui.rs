use crate::workers::{
    AudioControl, AudioWorker, Config, DeviceStatus, DeviceWorker, InputWorker, Sound, Worker,
};

use crate::{steno::*, *};
use evdev::{uinput, Key as VirtualKey};
use rodio::{
    source::{SineWave, Source},
    OutputStream, Sink,
};
use std::convert::TryInto;
use std::io::stdout;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::{
    event::Key as PhysicalKey,
    raw::{IntoRawMode, RawTerminal},
};
use tui::{backend::TermionBackend, layout::*, terminal::Frame, widgets::*, Terminal};
enum Event {
    Play,
    Pause,
    Stop,
}
pub struct WorkerPool {
    pub audio: AudioWorker,
    pub device: DeviceWorker,
    pub input: InputWorker,
}
pub struct Tui {
    terminal: Terminal<TermionBackend<RawTerminal<std::io::Stdout>>>,
    keyboard: uinput::VirtualDevice,
    worker_pool: WorkerPool,
    dictionary: Dictionary,
    output: History<Command, ListState>,
    last: History<String, TableState>,
    raw: History<String, ListState>,
}

impl Default for Tui {
    fn default() -> Self {
        let dictionary = Dictionary::from_file("./main.json");
        let backend = TermionBackend::new(stdout().into_raw_mode().unwrap());
        let mut terminal = tui::Terminal::new(backend).unwrap();
        let raw = History::new(Vec::new(), 10);
        let output = History::new(Vec::new(), 10);
        let last = History::new(Vec::new(), 1);
        let config = Config::default();
        let worker_pool = WorkerPool {
            audio: AudioWorker::start(config.clone()),
            device: DeviceWorker::start(config.clone()),
            input: InputWorker::default(),
        };
        let keyboard = uinput::VirtualDeviceBuilder::new()
            .unwrap()
            .name("wayplover")
            .with_keys(&VIRT_KEY_MAP)
            .unwrap()
            .build()
            .unwrap();
        terminal.clear().unwrap();
        Self {
            terminal,
            keyboard,
            raw,
            last,
            output,
            dictionary,
            worker_pool,
        }
    }
}

impl Tui {
    pub fn new(worker_pool: WorkerPool, dictionary: Dictionary) -> Self {
        let backend = TermionBackend::new(stdout().into_raw_mode().unwrap());
        let mut terminal = tui::Terminal::new(backend).unwrap();
        let output = History::new(Vec::new(), 500);
        let raw = History::new(Vec::new(), 500);
        let last = History::new(Vec::new(), 1);
        let keyboard = uinput::VirtualDeviceBuilder::new()
            .unwrap()
            .name("wayplover")
            .with_keys(&VIRT_KEY_MAP)
            .unwrap()
            .build()
            .unwrap();
        terminal.clear().unwrap();
        Self {
            terminal,
            keyboard,
            raw,
            last,
            output,
            dictionary,
            worker_pool,
        }
    }
    fn handle_strokes(&mut self, stroke: Stroke) {
        use evdev::*;
        let mut seq = Vec::new();
        let (sym, text) = {
            let s = stroke.resolve(&mut self.dictionary);
            match s.clone() {
                Command::Error(_) => self
                    .worker_pool
                    .audio
                    .send(AudioControl::Play(Sound::Error)),
                Command::Output(text) => self
                    .worker_pool
                    .audio
                    .send(AudioControl::Speak(text.clone())),
                _ => {}
            }
            s.as_text()
        };

        let text_stream: Vec<Option<&(Option<VirtualKey>, VirtualKey)>> =
            text.split("").map(|c| KEY_CODE.get(c)).collect();
        match sym {
            ActionSymbol::Suffix => {
                seq.push(VirtualKey::KEY_BACKSPACE);
            }
            ActionSymbol::Delete => {
                seq.extend(vec![VirtualKey::KEY_LEFTCTRL, VirtualKey::KEY_BACKSPACE])
            }
            _ => {}
        }
        for key_combo in text_stream {
            if let Some(combo) = key_combo {
                match combo {
                    (Some(modifier), letter) => {
                        seq.extend(vec![modifier, letter]);
                    }
                    (None, letter) => {
                        seq.push(*letter);
                    }
                }
            }
        }
        let mut held: (bool, Option<VirtualKey>) = (false, None);
        let len = seq.len();

        for (i, &key) in seq.iter().enumerate() {
            let down = InputEvent::new(EventType::KEY, key.code(), 1);
            let up = InputEvent::new(EventType::KEY, key.code(), 0);
            match key {
                Key::KEY_LEFTCTRL | Key::KEY_LEFTALT | Key::KEY_LEFTSHIFT => {
                    self.keyboard.emit(&[down]).unwrap();
                    sleep(Duration::from_millis(1));
                    held = (true, Some(key));
                }
                _ => {
                    self.keyboard.emit(&[down, up]).unwrap();
                    sleep(Duration::from_millis(1));
                    if let (true, Some(key)) = held {
                        self.keyboard
                            .emit(&[InputEvent::new(EventType::KEY, key.code(), 0)])
                            .unwrap();
                        sleep(Duration::from_millis(1));
                    } else if i.eq(&(len - 1)) {
                        let down = InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 1);
                        let up = InputEvent::new(EventType::KEY, Key::KEY_SPACE.code(), 0);
                        self.keyboard.emit(&[down, up]).unwrap();
                        sleep(Duration::from_millis(1));
                    }
                }
            }
        }
        self.output.push(stroke.resolve(&mut self.dictionary));
        self.output.select(0);
        self.raw.push(stroke.plain());
        self.raw.select(0);
        self.last.replace(stroke.raw())
    }

    fn handle_keys(&mut self, _key: PhysicalKey) {}

    pub fn run(&mut self) {
        use tui::widgets::*;
        loop {
            self.terminal.get_frame().set_cursor(0, 0);
            if let Some(status) = self.worker_pool.device.recv() {
                match status {
                    DeviceStatus::Input(s) => {
                        self.handle_strokes(s);
                    }
                }
            }
            let mut chord_history = self.output.clone();
            let mut stroke_history = self.raw.clone();
            let last_stroke = self.last.clone();
            self.terminal
                .draw(|f| {
                    let size = f.size();
                    let segments = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints(
                            [
                                Constraint::Percentage(70),
                                Constraint::Ratio(1, 15),
                                Constraint::Percentage(20),
                            ]
                            .as_ref(),
                        )
                        .split(size);

                    {
                        /*
                         * Chord History
                         * */
                        let max = chord_history.max_size.clone();
                        let chords: Vec<ListItem> = chord_history
                            .items
                            .iter()
                            .map(|cmd: &steno::Command| {
                                use steno::Command::*;
                                use tui::style::*;
                                match cmd {
                                    Append(suffix) => ListItem::new(suffix.as_str())
                                        .style(Style::default().fg(Color::Yellow)),
                                    Output(text) => ListItem::new(text.as_str()),
                                    Error(stroke) => ListItem::new(stroke.as_str())
                                        .style(Style::default().fg(Color::LightRed)),
                                    Delete => ListItem::new("*")
                                        .style(Style::default().fg(Color::LightCyan)),
                                }
                            })
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
                    let mut widths: Vec<Constraint> = Vec::new();
                    for _ in 0..STENO_ORDER.len() {
                        widths.push(Constraint::Ratio(1, STENO_ORDER.len().try_into().unwrap()));
                    }
                    f.render_widget(Self::draw_last(&last_stroke).widths(&widths), segments[1]);
                    f.render_stateful_widget(
                        Self::draw_histroy(&stroke_history),
                        segments[2],
                        stroke_history.state(),
                    );
                })
                .unwrap();
            //self.terminal.get_frame().set_cursor(1, 1);
            self.terminal.autoresize().unwrap();
        }
    }

    fn draw_histroy(history: &History<String, ListState>) -> List<'static> {
        let max = history.max_size;
        let strokes: Vec<ListItem> = history
            .clone()
            .items
            .iter()
            .map(|i| {
                let s = i
                    .split("")
                    .map(|s| format!("{}", s))
                    .collect::<Vec<String>>();
                let s = s.join("");
                ListItem::new(s)
            })
            .collect();
        let window = Block::default()
            .title(format!("Stroke History: {}/{}", strokes.len(), max))
            .borders(Borders::all());
        List::new(strokes.into_iter().rev().collect::<Vec<ListItem>>())
            .block(window)
            .highlight_symbol(">>")
    }

    fn draw_last(stroke: &History<String, TableState>) -> Table {
        use tui::style::*;
        use tui::widgets::*;
        let steno_order = STENO_ORDER.descending_keys();
        let s = stroke.items.clone();
        let cells = steno_order
            .rev()
            .map(|letter| {
                if s.contains(&letter.to_string()) {
                    Cell::from(format!("{}", letter.replace("-", "")))
                        .style(Style::default().fg(Color::White))
                } else {
                    Cell::from(format!("{}", letter.replace("-", "")))
                        .style(Style::default().fg(Color::Blue))
                }
            })
            .collect::<Vec<Cell>>();
        let header = Row::new(cells);

        Table::new(vec![])
            .header(header)
            .block(Block::default().title("Steno Order").borders(Borders::ALL))
            .column_spacing(1)
            .style(Style::default().fg(Color::White).bg(Color::Black))
    }

    fn draw_output() {}
}
#[derive(Clone)]
struct History<T, S> {
    items: Vec<T>,
    max_size: usize,
    pub state: S,
}
trait Selectable<S> {
    fn select(&mut self, i: usize);
    fn unselect(&mut self);
}
impl<T: Clone, S: Default> History<T, S> {
    fn new(items: Vec<T>, max_size: usize) -> Self {
        Self {
            items,
            max_size,
            state: S::default(),
        }
    }
    pub fn push(&mut self, items: T) {
        if self.items.len() > self.max_size {
            self.items.clear();
        }
        self.items.push(items);
    }
    pub fn replace(&mut self, items: Vec<T>) {
        self.items.clear();
        self.items.extend(items);
    }
    pub fn state(&mut self) -> &mut S {
        &mut self.state
    }
}
impl<T: Clone> Selectable<ListState> for History<T, ListState> {
    fn select(&mut self, i: usize) {
        self.state().select(Some(i));
    }
    fn unselect(&mut self) {
        self.state().select(None);
    }
}
impl<T: Clone> Selectable<TableState> for History<T, TableState> {
    fn select(&mut self, i: usize) {
        self.state().select(Some(i));
    }
    fn unselect(&mut self) {
        self.state().select(None);
    }
}
