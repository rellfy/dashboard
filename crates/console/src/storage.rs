use std::{fs, io};
use std::path::{PathBuf};
use serde::{Serialize, Deserialize};
use api::mail::Mailbox;
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

impl Storage {
    pub fn get_mailbox_by_id(&self, id: &str) -> Option<&OutlookMailbox> {
        self.outlook.iter().find(|mailbox| mailbox.get_id() == id)
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
    ).expect("storage::set: failed to write to storage");
}

pub fn get() -> Storage {
    fn read() -> io::Result<String> {
        fs::read_to_string(get_storage_path())
    }
    let mut storage_string = read();
    if storage_string.is_err() {
        set(&Storage::default());
        storage_string = read();
    }
    serde_json::from_str(&storage_string.unwrap())
        .expect("storage::get: could not deserialize storage")
}
