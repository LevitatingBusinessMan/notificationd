
#[derive(Debug, Clone)]
pub struct NotificationDetails {
    pub id: Option<usize>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub tags: Vec<String>,
    pub user: Option<String>,
    pub timestamp: Option<String>,
}

impl NotificationDetails {
    pub fn new() -> Self {
        Self {
            id: None,
            user: None,
            title: None,
            body: None,
            tags: vec![],
            timestamp: None,
        }
    }
}
