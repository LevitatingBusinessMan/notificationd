use anyhow::anyhow;
use rusqlite::Connection;
use rusqlite::params;

use notificationd::notifications::NotificationDetails;

pub fn setup_database(db: &mut Connection) -> rusqlite::Result<usize> {
    let n = db.execute(
        "CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user TEXT NOT NULL,
                title TEXT,
                body TEXT,
                tags TEXT,
                timestamp INTEGER NOT NULL
        )",
        (),
    )?;
    Ok(n)
}

pub trait NotificationDetailsDatabaseExt
where
    Self: Sized,
{
    type Key;
    fn save(&self, db: &mut Connection) -> anyhow::Result<usize>;
    fn load(db: &mut Connection, key: Self::Key) -> rusqlite::Result<Self>;
    fn load_all(db: &mut Connection, limit: Option<u32>) -> rusqlite::Result<Vec<Self>>;
}

impl NotificationDetailsDatabaseExt for NotificationDetails {
    type Key = u32;

    fn save(&self, db: &mut Connection) -> anyhow::Result<usize> {
        let user = self
            .user
            .as_ref()
            .ok_or(anyhow!("No user on notification"))?;
        Ok(db.execute(
            "INSERT INTO notifications (user, title, body, tags, timestamp)
            VALUES (?1, ?2, ?3, ?4, unixepoch())",
            params![user, self.title, self.body, self.tags.join(" ")],
        )?)
    }

    fn load(db: &mut Connection, key: Self::Key) -> rusqlite::Result<Self> {
        todo!()
    }

    fn load_all(db: &mut Connection, limit: Option<u32>) -> rusqlite::Result<Vec<Self>> {
        let mut stmt = db.prepare(
            "SELECT * FROM (
                SELECT id, user, title, body, tags, datetime(timestamp, 'unixepoch') as timestamp
                FROM notifications ORDER BY id DESC LIMIT ?1
            ) ORDER BY id ASC
        ")?;
        stmt.query_map([limit.map_or(-1, |l| l as i32)], |row| {
            Ok(NotificationDetails {
                id: row.get(0)?,
                user: row.get(1)?,
                title: row.get(2)?,
                body: row.get(3)?,
                tags: row
                    .get::<usize, String>(4)?
                    .split(" ")
                    .filter_map(|s| {
                        if !s.is_empty() {
                            Some(String::from(s))
                        } else {
                            None
                        }
                    })
                    .collect(),
                timestamp: row.get(5)?,
            })
        })?
        .into_iter()
        .collect()
    }
}
