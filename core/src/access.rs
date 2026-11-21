#[derive(Clone, PartialEq, Eq)]
pub struct BoardAccess {
    url: String,
    token: String,
}

impl BoardAccess {
    pub fn new(url: impl Into<String>, token: impl Into<String>) -> BoardAccess {
        BoardAccess {
            url: url.into(),
            token: token.into(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub(crate) fn token(&self) -> &str {
        &self.token
    }
}
