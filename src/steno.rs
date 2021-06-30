use crate::*;
pub struct Stroke {
    pub keys: Vec<String>,
}
impl Stroke {
    pub fn new(steno_keys: Vec<&str>) -> Self {
        let mut key_set = HashSet::new();
        for key in steno_keys {
            key_set.insert(key.into());
        }
        let mut steno_vec = key_set.into_iter().collect::<Vec<String>>();

        steno_vec.sort_by(|k1, k2| {
            let (k1, k2) = (STENO_ORDER.get(k1.as_str()), STENO_ORDER.get(k2.as_str()));
            let (k1, k2) = (k1.unwrap(), k2.unwrap());
            k1.cmp(k2)
        });
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
    pub fn resolve(&self, dict: &mut Dictonary) -> String {
        dict.lookup(
            self.keys
                .iter()
                .map(|e| e.replace("-", ""))
                .collect::<Vec<String>>()
                .join(""),
        )
        .unwrap_or("".to_string())
    }
    pub fn plain(&self) -> String {
        self.keys
            .iter()
            .map(|e| e.replace("-", ""))
            .collect::<Vec<String>>()
            .join("")
    }
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}
pub struct Dictonary {
    pub repr: HashMap<String, String>,
    pub last: String,
}
impl Dictonary {
    pub fn from_file(path: &str) -> Self {
        let mut file = File::open(path).unwrap();
        let mut contents = std::string::String::new();
        let bytes_read = file.read_to_string(&mut contents);
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
                    last: std::string::String::new(),
                }
            }
            _ => {
                panic!();
            }
        }
    }
    fn lookup(&mut self, steno_string: String) -> Option<String> {
        let mut entrys: Vec<&String> = self
            .repr
            .keys()
            .into_iter()
            .filter(|&s| s.contains(&steno_string))
            .collect();
        let mut output: String = steno_string.clone();
        entrys.sort_by(|&a, &b| Ord::cmp(&a.len(), &b.len()));
        for stroke in entrys {
            if stroke.eq(&steno_string) {
                output = self.repr.get(stroke).unwrap().to_owned();
            }
        }
        return Some(output);
    }
}
