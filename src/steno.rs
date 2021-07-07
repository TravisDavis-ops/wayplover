use crate::*;
use regex::*;
use std::borrow::*;

#[derive(Debug)]
pub struct Stroke {
    pub keys: Vec<String>,
}
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
impl Stroke {
    pub fn new(steno_keys: Vec<&str>) -> Self {
        let mut key_set = HashSet::new();
        for key in steno_keys {
            key_set.insert(key.into());
        }
        let mut steno_vec = key_set.into_iter().collect::<Vec<String>>();

        steno_vec.sort_by(|k1, k2| STENO_ORDER.compare( k1.as_str(), k2.as_str()));

        if steno_vec.contains(&"#".to_string()) {
            let mut number_steno_vec = Vec::new();
            for (i, l) in steno_vec.iter_mut().enumerate() {
                if let Some(&number) = STENO_NUMBERS.get(l.as_str()) {
                    number_steno_vec[i] = number.to_string();
                }
            }
            return Self {
                keys: number_steno_vec,
            };
        }
        //let steno_vec = steno_vec.iter_mut().map(|e| e.replace("-", "")).collect();
        Self { keys: steno_vec }
    }
    pub fn resolve(&self, dict: &mut Dictionary) -> Command {
        dict.lookup(
            self.keys
                .iter()
                .map(|e| e.replace("-", ""))
                .collect::<Vec<String>>()
                .join(""),
        )
    }
    pub fn plain(&self) -> String {
        self.keys
            .iter()
            .map(|e| e.replace("-", ""))
            .collect::<Vec<String>>()
            .join("")
    }
    pub fn raw(&self) -> Vec<String> {
        self.keys.clone()
    }
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}
pub struct Dictionary {
    pub repr: HashMap<String, String>,
    pub last: Option<String>,
}
impl Dictionary {
    pub fn from_file(path: &str) -> Self {
        let mut file = File::open(path).unwrap();
        let mut contents = std::string::String::new();
        let _bytes_read = file.read_to_string(&mut contents);
        let json = json::parse(&contents).unwrap();
        use json::JsonValue::*;
        match json {
            Object(mut obj) => {
                let mut map = HashMap::new();
                for (key, value) in obj.iter_mut() {
                    map.insert(key.to_string(), value.to_string());
                }
                Self {
                    repr: map,
                    last: None,
                }
            }
            _ => {
                panic!();
            }
        }
    }


    fn lookup(&mut self, chord: String) -> Command {
        if chord.eq(&"*".to_string()) {
            return Command::Delete;
        }
            // i need to get a handle on this dictonary problem

        if let Some(stroke) = self.repr.get(&chord) {
            let re = Regex::new(r"\{\^(?P<suffix>\w*)\}").unwrap();
            let captures = re.captures(&stroke);
            if let Some(cap) = captures {
                if let Some(last) = self.last.clone() {
                    return Command::Append(format!("{}{}", last , &cap["suffix"]));
                }
                return Command::Output(cap["suffix"].to_string());
            } else {
                self.last = Some(stroke.to_owned());
                return Command::Output(stroke.to_owned());
            }
        }
        self.last = None;

        return Command::Error(chord);
    }
}
