use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PushResponse<T>
where
    T: std::fmt::Debug,
{
    pub data: Vec<PushReceipt<T>>,
}

/// See [the Expo documentation on Push Notifications] for details about the push notifications response from the server.
///
/// [the Expo documentation on Push Notifications]: https://docs.expo.io/versions/latest/guides/push-notifications#response-format
#[derive(Debug, Deserialize, PartialEq)]
pub struct PushReceipt<T>
where
    T: std::fmt::Debug,
{
    pub status: String,
    pub message: Option<String>,
    pub details: Option<T>,
}
