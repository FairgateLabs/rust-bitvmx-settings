use serde::Deserialize;
use std::{fmt, ops::Deref};

#[derive(Clone, Deserialize)]
pub struct Secret(String);

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "************")
    }
}

impl Deref for Secret {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Secret {
    fn from(s: String) -> Self {
        Secret(s)
    }
}

impl From<&str> for Secret {
    fn from(s: &str) -> Self {
        Secret(s.to_owned())
    }
}
