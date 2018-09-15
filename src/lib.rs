extern crate rand;
extern crate walkdir;

use std::io::{BufRead, BufReader};
use rand::Rng;
use rand::RngCore;
use walkdir::WalkDir;
use std::fs::File;
use std::collections::{BTreeSet, BTreeMap};

#[derive(Debug, Default)]
pub struct TCPList {
    pub types: BTreeMap<TCPType, Vec<String>>,
    pub conditions: Vec<String>,
    pub modifiers: Vec<String>,
    pub anomalies: Vec<String>,
}

#[derive(Debug, Default)]
pub struct TCP {
    pub types: Vec<(TCPType, String)>,
    pub conditions: Vec<String>,
    pub modifiers: Vec<String>,
    pub anomalies: Vec<String>,
    pub designer: bool,
}

impl Display for TCP {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        if self.designer {
            fmt.write_str("designer ")?;
        }
        fmt.write_str(&self.types.iter().map(|(category, name)| format!("{}{}", category.get_emoji(), name)).collect::<Vec<_>>().join("/"))?;
        if self.conditions.len() > 0 {
            fmt.write_str(", conditions: ")?;
            fmt.write_str(&self.conditions.join(", "))?;
        }
        if self.anomalies.len() > 0 {
            fmt.write_str(", anomalies: ")?;
            fmt.write_str(&self.anomalies.join(", "))?;
        }
        if self.modifiers.len() > 0 {
            fmt.write_str(", modifiers: ")?;
            fmt.write_str(&self.modifiers.join(", "))?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum TCPType {
    Abstract,
    Body,
    Creature,
    Food,
    Machine,
    Nature,
    Form,
    Storage,
    Weapon,
    Unknown,
}

use TCPType::{Abstract, Body, Creature, Food, Machine, Nature, Form, Storage, Weapon, Unknown};

impl TCPType {
    fn get_emoji(&self) -> &'static str {
        match self {
            Abstract => "🎭",
            Body => "👁️",
            Creature => "🐈",
            Food => "🌶",
            Machine => "⚙️",
            Nature => "☁️",
            Form => "⚪",
            Storage => "📦",
            Weapon => "🗡️",
            Unknown => "❓",
        }
    }
}

use std::fmt::Display;
use std::fmt::Formatter;

impl Display for TCPType {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        Display::fmt(match *self {
            Abstract => "Abstract",
            Body => "Body",
            Creature => "Creature",
            Food => "Food",
            Machine => "Machine",
            Nature => "Nature",
            Form => "Form",
            Storage => "Storage",
            Weapon => "Weapon",
            Unknown => "Unknown",
        },
                     formatter)
    }
}

impl From<u32> for TCPType {
    fn from(n: u32) -> Self {
        match n {
            0 => Abstract,
            1 => Body,
            2 => Creature,
            3 => Food,
            4 => Machine,
            5 => Nature,
            6 => Form,
            7 => Storage,
            8 => Weapon,
            _ => Unknown,
        }
    }
}

impl<'a> From<&'a str> for TCPType {
    fn from(s: &'a str) -> Self {
        match s {
            "abstract" => Abstract,
            "body" => Body,
            "creature" => Creature,
            "food" => Food,
            "machine" => Machine,
            "nature" => Nature,
            "form" => Form,
            "storage" => Storage,
            "weapon" => Weapon,
            _ => Unknown,
        }
    }
}

impl TCPList {
    pub fn new(root: &str) -> TCPList {
        let mut types = BTreeMap::new();
        types.insert(Unknown, BTreeSet::new());
        let mut lists = Vec::new();
        let mut set = BTreeSet::new();
        for path in WalkDir::new(&format!("{}/{}", root, "types")) {
            let path = path.unwrap();
            let path = path.path();
            if path.is_file() {
                let file = File::open(path).expect(&format!("Couldn't read {:?}", path));
                let mut lines = BufReader::new(&file).lines();
                let mut current_type = TCPType::Unknown;
                while let Some(Ok(line)) = lines.next() {
                    let line = line.trim().to_string();
                    if !line.is_empty() {
                        if line.as_bytes()[0] == b'#' {
                            current_type = TCPType::from(&line[1..]);
                            if !types.contains_key(&current_type) {
                                types.insert(current_type, BTreeSet::new());
                            }
                        } else if !set.contains(&line) {
                            set.insert(line.clone());
                            types.get_mut(&current_type).unwrap().insert(line);
                        }
                    }
                }
            }
        }

        if types.get(&Unknown).unwrap().len() == 0 {
            types.remove(&Unknown);
        }

        for dir in &["conditions", "modifiers", "anomalies"] {
            let mut set = BTreeSet::new();
            for path in WalkDir::new(&format!("{}/{}", root, dir)) {
                let path = path.unwrap();
                let path = path.path();
                if path.is_file() {
                    let file = File::open(path).expect(&format!("Couldn't read {:?}", path));
                    let mut lines = BufReader::new(&file).lines();
                    while let Some(Ok(line)) = lines.next() {
                        let line = line.trim().to_string();
                        if !line.is_empty() {
                            set.insert(line);
                        }
                    }
                }
            }
            lists.push(set.into_iter().collect());
        }
        TCPList {
            types: types.into_iter().map(|(key, value)| (key, value.into_iter().collect())).collect(),
            conditions: lists.remove(0),
            modifiers: lists.remove(0),
            anomalies: lists.remove(0),
        }
    }

    pub fn gen(&self) -> TCP {
        let mut random = rand::thread_rng();
        let designer = random.gen_bool(0.25);
        if random.next_u32() & 1 == 0 {
            let (tcp_type, list) = self.types
                .iter()
                .skip(random.next_u32() as usize % self.types.len())
                .next()
                .unwrap();
            TCP {
                types: vec![(*tcp_type, random.choose(list).unwrap().clone())],
                designer,
                ..Default::default()
            }
        } else {
            let mut type_count = 1;
            let mut condition_count = 0;
            let mut modifier_count = 0;
            let mut anomaly_count = 0;
            for _ in 0..4 {
                if random.next_u32() % 100 > 86 {
                    type_count += 1;
                }
                if random.next_u32() % 100 > 94 {
                    condition_count += 1;
                }
                if random.next_u32() % 100 > 86 {
                    modifier_count += 1;
                }
                if random.next_u32() % 100 > 76 {
                    anomaly_count += 1;
                }
            }
            let mut types = Vec::new();
            let mut type_map = BTreeMap::new();
            for _ in 0..type_count {
                let (tcp_type, list) = self.types
                    .iter()
                    .skip(random.next_u32() as usize % self.types.len())
                    .next()
                    .unwrap();
                let list = type_map.entry(tcp_type).or_insert_with(|| list.clone());
                let index = random.next_u32() as usize % list.len();
                types.push((*tcp_type, list.swap_remove(index)));
            }
            TCP {
                types,
                conditions: rand::seq::sample_iter(&mut random, self.conditions.clone(), condition_count).unwrap(),
                modifiers: rand::seq::sample_iter(&mut random, self.modifiers.clone(), modifier_count).unwrap(),
                anomalies: rand::seq::sample_iter(&mut random, self.anomalies.clone(), anomaly_count).unwrap(),
                designer,
            }
        }
    }
}

