use std::sync::LazyLock;

enum Message {
    Login(String, String)
}

impl TryFrom<&str> for Message {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}
