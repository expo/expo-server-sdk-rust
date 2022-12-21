#[derive(Debug, thiserror::Error)]
pub enum ExpoNotificationError {
    #[error("network request error: {0}")]
    Request(reqwest::Error),
    #[error("IO error: {0}")]
    Io(std::io::Error),
    #[error("nothing to send")]
    Empty,
}

impl From<reqwest::Error> for ExpoNotificationError {
    fn from(value: reqwest::Error) -> Self {
        Self::Request(value)
    }
}
impl From<std::io::Error> for ExpoNotificationError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
