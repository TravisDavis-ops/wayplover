use crate::{steno::*, workers::InputWorker, *};
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
use tui::{backend::TermionBackend, widgets::*, Terminal, terminal::Frame, layout::*};
enum Event {
    Play,
    Pause,
    Stop,
}
pub struct Ui {
    terminal: Terminal<TermionBackend<RawTerminal<std::io::Stdout>>>,
    keyboard: uinput::VirtualDevice,
    worker: InputWorker,
    dictionary: Dictionary,
    audio_thread: (mpsc::Sender<Event>, thread::JoinHandle<()>),
    chord_history: History<Command, ListState>,
    last_stroke: History<String, TableState>,
    stroke_history: History<String, ListState>,
}

impl Default for Ui {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let audio_thread = {
            thread::Builder::new()
                .name("Audio Server".to_string())
                .spawn(move || {
                    let (stream, stream_handle) = OutputStream::try_default().unwrap();
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    loop {
                        match rx.recv() {
                            Ok(Event::Play) => {
                                let sound =
                                    SineWave::new(440).take_duration(Duration::from_secs(3));
                                sink.append(sound);
                            }
                            _ => {
                                sink.stop();
                            }
                        }
                        sink.sleep_until_end();
                    }
                })
                .unwrap()
        };
        let dictionary = Dictionary::from_file("./main.json");
        let backend = TermionBackend::new(stdout().into_raw_mode().unwrap());
        let mut terminal = tui::Terminal::new(backend).unwrap();
        let stroke_history = History::new(Vec::new(), 10);
        let chord_history = History::new(Vec::new(), 10);
        let last_stroke = History::new(Vec::new(), 1);
        let worker = InputWorker::default();
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
            stroke_history,
            chord_history,
            dictionary,
            audio_thread: (tx, audio_thread),
            last_stroke,
            worker,
        }
    }
}

impl Ui {
    pub fn new(worker: InputWorker, dictionary: Dictionary) -> Self {
        let backend = TermionBackend::new(stdout().into_raw_mode().unwrap());
        let mut terminal = tui::Terminal::new(backend).unwrap();
        let chord_history = History::<steno::Command, ListState>::new(
            Vec::new(),
            terminal.size().unwrap().height as usize,
        );
        let stroke_history =
            History::<String, ListState>::new(Vec::new(), terminal.size().unwrap().height as usize);
        let last_stroke = History::<String, _>::new(Vec::new(), 1);
        let keyboard = uinput::VirtualDeviceBuilder::new()
            .unwrap()
            .name("wayplover")
            .with_keys(&VIRT_KEY_MAP)
            .unwrap()
            .build()
            .unwrap();
        let (tx, rx) = mpsc::channel();
        let audio_thread = {
            thread::Builder::new()
                .name("Audio Server".to_string())
                .spawn(move || {
                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    loop {
                        match rx.recv() {
                            Ok(Event::Play) => {
                                let sound =
                                    SineWave::new(440).take_duration(Duration::from_millis(5));
                                sink.append(sound);
                            }
                            _ => {
                                sink.stop();
                            }
                        }
                        sink.sleep_until_end();
                    }
                })
                .unwrap()
        };
        terminal.clear().unwrap();
        Self {
            terminal,
            keyboard,
            audio_thread: (tx, audio_thread),
            last_stroke,
            chord_history,
            stroke_history,
            dictionary,
            worker,
        }
    }
    fn handle_strokes(&mut self, stroke: Stroke) {
        use evdev::*;
        let mut seq = Vec::new();
        let (sym, text) = {
            let s = stroke.resolve(&mut self.dictionary);
            match s {
                Command::Error(_) => {
                    self.audio_thread.0.send(Event::Play).unwrap();
                }
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
        self.chord_history.push(stroke.resolve(&mut self.dictionary));
        self.chord_history.select(0);
        self.stroke_history.push(stroke.plain());
        self.stroke_history.select(0);
        self.last_stroke.replace(stroke.raw())
    }

    fn handle_keys(&mut self, _key: PhysicalKey) {}

    pub fn run(&mut self) {
        use crate::workers::InputEvents::*;
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
            let mut last_stroke = self.last_stroke.clone();
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
                        f.render_widget(Self::draw_last(&last_stroke).widths(&widths),segments[1]);
                    f.render_stateful_widget(Self::draw_histroy(&stroke_history), segments[2], stroke_history.state());
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
