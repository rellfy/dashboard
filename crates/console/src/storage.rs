use std::{fs, io};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use api::outlook::OutlookMailbox;

const STORAGE_FILE_NAME: &'static str = "dashboard.json";

#[derive(Serialize, Deserialize)]
pub struct Storage {
    pub outlook: Vec<OutlookMailbox>,
}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            outlook: vec![]
        }
    }
}

fn get_storage_path() -> PathBuf {
    dirs::config_dir().unwrap().join(STORAGE_FILE_NAME)
}

pub fn set(storage: &Storage) {
    fs::write(
        get_storage_path(),
        serde_json::to_string(storage)
            .expect("storage::set: could not serialize storage before saving")
    );
}

pub fn get() -> Storage {
    fn read() -> io::Result<String> { fs::read_to_string(get_storage_path()) };
    let mut storage_string = read();
    if storage_string.is_err() {
        set(&Storage::default());
        storage_string = read();
    }
    serde_json::from_str(&storage_string.unwrap())
        .expect("storage::get: could not deserialize storage")
}
