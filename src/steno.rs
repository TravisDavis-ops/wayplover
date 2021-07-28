use crate::*;
use diesel::prelude::*;
use regex::*;
use std::borrow::*;
#[derive(Debug)]
pub struct Chord(Vec<String>);
#[derive(Clone)]
pub enum Command {
    Append(String),
    Output(String),
    Error(String),
    Delete,
}

pub enum ActionSymbol {
    Suffix,
    Glue,
    Delete,
    Noop,
}
impl Command {
    pub fn as_text(&self) -> (ActionSymbol, String) {
        match self {
            Self::Append(s) => (ActionSymbol::Suffix, s.to_owned()),
            Self::Output(s) => (ActionSymbol::Noop, s.to_owned()),
            Self::Delete => (ActionSymbol::Delete, String::new()),
            Self::Error(s) => (ActionSymbol::Noop, s.to_owned()),
        }
    }
}
impl Chord {
    pub fn new(steno_keys: Vec<&str>) -> Self {
        let mut key_set = HashSet::new();
        for key in steno_keys {
            key_set.insert(key.into());
        }
        let mut steno_vec = key_set.into_iter().collect::<Vec<String>>();

        steno_vec.sort_by(|k1, k2| STENO_ORDER.compare(k1.as_str(), k2.as_str()));

        if steno_vec.contains(&"#".to_string()) {
            let mut number_steno_vec = Vec::new();
            for (i, l) in steno_vec.iter_mut().enumerate() {
                if let Some(&number) = STENO_NUMBERS.get(l.as_str()) {
                    number_steno_vec[i] = number.to_string();
                }
            }
            return Self(number_steno_vec);
        }
        //let steno_vec = steno_vec.iter_mut().map(|e| e.replace("-", "")).collect();
        Self(steno_vec)
    }
    pub fn resolve(&self, dict: &mut Dictionary) -> Command {
        dict.lookup(
            self.0
                .iter()
                .map(|e| {
                    let mut e = Vec::from(e.split("").collect::<Vec<&str>>());
                    if e[0] == "-" {
                        for i in 1..e.len() {
                            if e[i] == "-" {
                                e[i] = "";
                            }
                        }
                        e.join("")
                    } else {
                        e.join("").replace("-", "")
                    }
                })
                .collect::<Vec<String>>()
                .join(""),
        )
    }
    pub fn plain(&self) -> String {
        self.0
            .iter()
            .map(|e| e.replace("-", ""))
            .collect::<Vec<String>>()
            .join("")
    }
    pub fn raw(&self) -> Vec<String> {
        self.0.clone()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
use sm::sm;
use sm::NoneEvent;
sm! {
    Translator {
        InitialStates { Idle, Waiting, Finished}
        NeedMore {
            Idle => Waiting
                Waiting => Waiting
        }
        Finish {
            Waiting => Finished
        }
    }
}
use Translator::*;
pub struct Dictionary {
    conn: SqliteConnection,
    path: String,
    last: Vec<String>,
}
impl Clone for Dictionary {
    fn clone(&self) -> Self {
        let conn = SqliteConnection::establish(self.path.as_str()).unwrap();
        Self{conn, last: self.last.clone(), path: self.path.clone()}
    }
}
impl Dictionary {
    pub fn from_file(path: &str) -> Self {
        let conn = SqliteConnection::establish(path).unwrap();
        Self {
            conn,
            path: path.to_string(),
            last: Vec::new(),
        }
    }

    fn lookup(&mut self, in_chord: String) -> Command {
        if in_chord.eq(&"*".to_string()) {
            return Command::Delete;
        }
        self.last.push(in_chord.clone());
        use crate::schema::dictionary::dsl::*;
        let entry = dictionary
            .filter(chord.eq(&in_chord))
            .order_by(id.desc())
            .first::<models::Entry>(&self.conn);

        match entry {
            Ok(e) => {
                info!("Chord: {}, ({}, {})", in_chord, e.chord, e.translation);
                self.last.clear();
                return Command::Output(e.translation);
            }
            Err(_) => {
                return Command::Error(in_chord);
            }
        }
    }

    pub fn find(&self, search: &String) -> Vec<models::Entry> {
        use crate::schema::dictionary::dsl::*;
        dictionary
            .filter(chord.like(format!("%{}%", search)))
            .order_by(id.asc())
            .load::<models::Entry>(&self.conn)
            .unwrap()
    }
}
