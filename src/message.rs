use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

/// A PushToken must be of the format `ExpoPushToken[xxx]` or `ExponentPushToken[xxx]`.
#[derive(Debug, Serialize, Clone)]
pub struct PushToken(String);

impl<'de> Deserialize<'de> for PushToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TryFrom::try_from(s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("expect format `ExpoPushToken[xxx]` or `ExponentPushToken[xxx]` but given {0}")]
pub struct PushTokenParseError(String);

impl TryFrom<String> for PushToken {
    type Error = PushTokenParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if (s.starts_with("ExponentPushToken[") || s.starts_with("ExpoPushToken["))
            && s.ends_with("]")
        {
            Ok(PushToken(s))
        } else {
            Err(PushTokenParseError(s))
        }
    }
}

/// The delivery priority of the message. Specify "default" or omit this field
/// to use the default priority on each platform, which is "normal" on Android
/// and "high" on iOS.
///
/// On Android, normal-priority messages won't open network connections on
/// sleeping devices and their delivery may be delayed to conserve the battery.
/// High-priority messages are delivered immediately if possible and may wake
/// sleeping devices to open network connections, consuming energy.
///
/// On iOS, normal-priority messages are sent at a time that takes into account
/// power considerations for the device, and may be grouped and delivered in
/// bursts. They are throttled and may not be delivered by Apple. High-priority
/// messages are sent immediately. Normal priority corresponds to APNs priority
/// level 5 and high priority to 10.
///
/// See the Expo documentation on [message format] for more details.
///
/// [message format]: https://docs.expo.io/versions/latest/guides/push-notifications#message-format
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Priority {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "high")]
    High,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Default
    }
}

impl FromStr for Priority {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Priority, Self::Err> {
        serde_json::from_str(s)
    }
}

/// A sound to play when the recipient receives this notification. Specify
/// "default" to play the device's default notification sound, or omit this
/// field to play no sound.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Sound {
    #[serde(rename = "default")]
    Default,
}

impl Default for Sound {
    fn default() -> Self {
        Sound::Default
    }
}

impl FromStr for Sound {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Sound, Self::Err> {
        serde_json::from_str(s)
    }
}

/// A `PushMessage` struct modelled after the one listed [here]:
///
/// [here]: https://docs.expo.io/versions/latest/guides/push-notifications#message-format
///
/// ## Example:
///
/// ```
/// # use expo_server_sdk::message::*;
/// # use std::str::FromStr;
///
/// let token = PushToken::from_str("ExpoPushToken[my-token]").unwrap();
/// let mut msg = PushMessage::new(token).body("test notification");
/// ```
#[derive(Serialize, Clone)]
pub struct PushMessage {
    pub to: PushToken,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<Sound>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<u32>,
}

impl PushMessage {
    pub fn new(push_token: PushToken) -> PushMessage {
        PushMessage {
            to: push_token,
            data: None,
            title: None,
            body: None,
            sound: None,
            ttl: None,
            expiration: None,
            priority: None,
            badge: None,
        }
    }

    pub fn data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn sound(mut self, sound: Sound) -> Self {
        self.sound = Some(sound);
        self
    }

    pub fn ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn expiration(mut self, expiration: u32) -> Self {
        self.expiration = Some(expiration);
        self
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn badge(mut self, badge: u32) -> Self {
        self.badge = Some(badge);
        self
    }
}
