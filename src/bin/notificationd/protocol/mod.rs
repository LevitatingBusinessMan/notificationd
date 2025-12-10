pub mod parser;

enum Message {
    Login(String, String),
}

impl TryFrom<&str> for Message {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// Create a reply
pub fn reply(
    id: Option<u32>,
    success: bool,
    command: &str,
    arguments: Vec<&str>,
    trailing: Option<&str>,
) -> String {
    format!(
        "{}{}{}{}{}\r\n",
        if let Some(id) = id {
            format!("{} ", id)
        } else {
            String::new()
        },
        if success { "+" } else { "-" },
        command,
        if arguments.is_empty() {
            String::new()
        } else {
            format!(" {}", arguments.join(" "))
        },
        if let Some(trailing) = trailing {
            format!(" : {}", trailing)
        } else {
            String::new()
        }
    )
}
