use std::collections::HashMap;
use api::mail::{Mailbox, Message};
use crate::Storage;

pub struct State {
    pub is_loaded: bool,
    pub unread_messages: Vec<Message>,
    pub parsed_message_bodies: HashMap<String, String>,
    pub selected_message_index: usize,
    pub cursor_height: usize,
    pub should_view_message_body: bool,
    pub should_exit: bool,
    pub should_skip_render: bool,
}

impl State {
    pub fn set_selected_message_as_read(&mut self, storage: &Storage) {
        if self.unread_messages.len() == 0 {
            return;
        }
        let selected_message_id = self
            .unread_messages[self.selected_message_index].id.clone();
        let selected_message_mailbox_id = self
            .unread_messages[self.selected_message_index].mailbox_id.clone();
        self.unread_messages.remove(self.selected_message_index);
        self.parsed_message_bodies.remove(&selected_message_id);
        self.decrease_selected_message_index();
        let mailbox = storage
            .get_mailbox_by_id(selected_message_mailbox_id.as_str())
            .unwrap().clone();
        tokio::task::spawn(mailbox.set_as_read(selected_message_id));
    }

    pub fn decrease_selected_message_index(&mut self) {
        if self.selected_message_index > 0 {
            self.selected_message_index -= 1;
        }
    }

    pub fn increase_selected_message_index(&mut self) {
        let count = self.unread_messages.len();
        let min_index = if count == 0 { 0 } else { count - 1 };
        if self.selected_message_index < min_index {
            self.selected_message_index += 1;
        }
    }
}