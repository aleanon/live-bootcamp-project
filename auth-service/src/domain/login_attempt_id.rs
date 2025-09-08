use std::{fmt::Display, ops::Deref};

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(Uuid);

impl LoginAttemptId {
    pub fn new() -> Self {
        let id = uuid::Uuid::new_v4();
        LoginAttemptId(id)
    }

    pub fn parse(id: &str) -> Result<Self, uuid::Error> {
        Ok(LoginAttemptId(Uuid::parse_str(id)?))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId::new()
    }
}

impl Display for LoginAttemptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for LoginAttemptId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
