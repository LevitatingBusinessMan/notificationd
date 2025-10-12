use std::sync::atomic::AtomicU32;

#[derive(Debug)]
pub struct NotificationDetails {
    pub title: Option<String>,
    pub body: Option<String>,
    pub tags: Option<Vec<String>>,
    pub user: Option<String>,
}

impl NotificationDetails {
    pub fn new() -> Self {
        Self {
            user: None,
            title: None,
            body: None,
            tags: None,
        }
    }
}
