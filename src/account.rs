use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Account {
    Offline {
        username: String,
        uuid: String,
    },
    Microsoft {
        username: String,
        uuid: String,
        access_token: String,
    },
}

impl Account {
    pub fn offline(username: impl Into<String>) -> Self {
        Self::Offline {
            username: username.into(),
            uuid: Uuid::new_v4().to_string(),
        }
    }

    pub fn username(&self) -> &str {
        match self {
            Self::Offline { username, .. } | Self::Microsoft { username, .. } => username,
        }
    }

    pub fn uuid(&self) -> &str {
        match self {
            Self::Offline { uuid, .. } | Self::Microsoft { uuid, .. } => uuid,
        }
    }

    pub fn access_token(&self) -> &str {
        match self {
            Self::Offline { .. } => "",
            Self::Microsoft { access_token, .. } => access_token,
        }
    }
}
