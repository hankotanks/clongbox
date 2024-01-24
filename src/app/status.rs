use std::{borrow, fmt, mem};

use egui::{mutex::Mutex, Response};
use once_cell::sync::Lazy;

pub struct Message {
    pub content: String,
    pub id: egui::Id,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

enum Status {
    Hover(Message),
    Persistent(Message),
    None,
}

impl Default for Status {
    fn default() -> Self {
        Self::None
    }
}

static STATUS: Lazy<Mutex<Status>> = Lazy::new(Mutex::default);

pub fn set_on_hover<'a, C>(response: &Response, message: C)
    where C: Into<borrow::Cow<'a, str>> {

    if response.hovered() {
        let content: borrow::Cow<'_, str> = message.into();
        let content = String::from(content);

        let message = Message {
            content,
            id: response.id,
        };

        let _ = mem::replace(&mut *STATUS.lock(), Status::Hover(message));
    } else if matches!(&*STATUS.lock(), Status::Hover(msg) if msg.id == response.id) {
        let _ = mem::take(&mut *STATUS.lock());
    }
}

pub fn set<'a, C>(id: egui::Id, message: C)
    where C: Into<borrow::Cow<'a, str>> {
    
    if !matches!(&*STATUS.lock(), Status::Hover(_)) {
        let content: borrow::Cow<'_, str> = message.into();
        let content = String::from(content);

        let message = Message {
            content,
            id,
        };

        let _ = mem::replace(&mut *STATUS.lock(), Status::Persistent(message));
    }
}

pub fn get() -> Option<&'static Message> {
    match &*STATUS.lock() {
        Status::Hover(msg) | Status::Persistent(msg) => unsafe {
            mem::transmute::<Option<&Message>, Option<&'static Message>>(Some(msg))
        },
        _ => None
    }
}

pub fn clear() {
    if matches!(&*STATUS.lock(), Status::Persistent(_)) {
        let _ = mem::take(&mut *STATUS.lock());
    }
}