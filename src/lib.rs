//! # Expo Push Notification Rust Client
//!
//! The Expo Push Notification client provides a way for you to send push notifications to users of
//! your mobile app using the Expo push notification services. For more details on the Expo push
//! notification service, go [here]
//!
//! [here]: https://docs.expo.io/versions/latest/guides/push-notifications
//!
//! ## Example: Sending a push notification
//!
//! ```
//! extern crate expo_server_sdk;
//! use expo_server_sdk::*;
//! use std::str::FromStr;
//!
//! let token = PushToken::from_str("ExpoPushToken[my-token]").unwrap();
//! let mut msg = PushMessage::new(token).body("test notification");
//!
//! let push_notifier = PushNotifier::new().gzip_policy(GzipPolicy::Always);
//! let result = push_notifier.send_push_notification(&msg);
//!
//! if let Ok(result) = result {
//!     println!("Push Notification Response: \n \n {:#?}", result);
//! }
//! ```
#![recursion_limit = "1024"]

extern crate failure;

extern crate flate2;
extern crate reqwest;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use reqwest::header::{
    qitem, Accept, AcceptEncoding, ContentEncoding, ContentType, Encoding, Headers,
};
use std::str::FromStr;

use failure::Error;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::value::Value;
use std::io::prelude::*;

/// A PushToken must be of the format `ExpoPushToken[xxx]` or `ExponentPushToken[xxx]`.
#[derive(Debug, Serialize, Clone)]
pub struct PushToken(String);

impl FromStr for PushToken {
    type Err = String;

    fn from_str(s: &str) -> Result<PushToken, Self::Err> {
        if (s.starts_with("ExponentPushToken[") || s.starts_with("ExpoPushToken["))
            && s.ends_with("]")
        {
            Ok(PushToken(s.to_string()))
        } else {
            Err(format!("A PushToken must be of the format `ExpoPushToken[xxx]` or `ExponentPushToken[xxx]`. Was given: {}", s))
        }
    }
}

#[derive(Debug, Deserialize)]
struct PushResponse<T>
where
    T: std::fmt::Debug,
{
    data: Vec<PushReceipt<T>>,
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
/// extern crate expo_server_sdk;
/// use expo_server_sdk::*;
/// use std::str::FromStr;
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

/// The policy under which we will gzip the request body that is sent to the push notification servers
pub enum GzipPolicy {
    /// Gzip only if the body is greater than 1024 bytes
    ZipGreaterThan1024Bytes,

    /// Never Gzip the request body
    Never,

    /// Always Gzip the request body
    Always,
}

impl Default for GzipPolicy {
    fn default() -> Self {
        GzipPolicy::ZipGreaterThan1024Bytes
    }
}

/// The `PushNotifier` takes one or more `PushMessage` to send to the push notification server
///
/// ## Example:
///
/// ```
/// extern crate expo_server_sdk;
/// use expo_server_sdk::*;
/// use std::str::FromStr;
///
/// let token = PushToken::from_str("ExpoPushToken[my-token]").unwrap();
/// let mut msg = PushMessage::new(token).body("test notification");
///
/// let push_notifier = PushNotifier::new().gzip_policy(GzipPolicy::Always);
/// let result = push_notifier.send_push_notification(&msg);
/// ```
pub struct PushNotifier {
    pub url: String,
    pub pushes_per_request: usize,
    pub gzip_policy: GzipPolicy,
    client: reqwest::Client,
}

impl PushNotifier {
    pub fn new() -> PushNotifier {
        PushNotifier {
            url: "https://exp.host/--/api/v2/push/send".to_string(),
            pushes_per_request: 100,
            gzip_policy: GzipPolicy::default(),
            client: reqwest::Client::new(),
        }
    }

    /// Specify the URL to the push notification server
    /// Default is the Expo push notification server.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    /// Specify the number of push notifications to group together into one request
    /// Default is 100.
    pub fn with_pushes_per_request(mut self, pushes_per_request: usize) -> Self {
        self.pushes_per_request = pushes_per_request;
        self
    }

    /// Specify when gzip'ping the request body occurrs
    /// Default policy is to gzip when the request body exceeds 1024 bytes.
    pub fn gzip_policy(mut self, gzip_policy: GzipPolicy) -> Self {
        self.gzip_policy = gzip_policy;
        self
    }

    /// Sends a vector of `PushMessage` to the push notification server.
    pub fn send_push_notifications(
        &self,
        messages: &[PushMessage],
    ) -> Result<Vec<PushReceipt<Value>>, Error> {
        let iter = messages.chunks(self.pushes_per_request);
        let mut responses: Vec<PushReceipt<Value>> = Vec::new();
        for chunk in iter {
            let mut response = self.send_push_notifications_chunk(&self.url, &chunk)?;
            responses.append(&mut response);
        }
        Ok(responses)
    }

    /// Sends a single `PushMessage` to the push notification server.
    pub fn send_push_notification(
        &self,
        message: &PushMessage,
    ) -> Result<PushReceipt<Value>, Error> {
        let mut result = self.send_push_notifications_chunk(&self.url, &[message.clone()])?;
        Ok(result.pop().unwrap())
    }

    fn send_push_notifications_chunk(
        &self,
        url: &str,
        messages: &[PushMessage],
    ) -> Result<Vec<PushReceipt<Value>>, Error> {
        let body = serde_json::to_string(&messages).unwrap();
        let should_compress = match self.gzip_policy {
            GzipPolicy::Always => true,
            GzipPolicy::Never => false,
            GzipPolicy::ZipGreaterThan1024Bytes => {
                if body.len() > 1024 {
                    true
                } else {
                    false
                }
            }
        };
        let mut res = self.request_async(url, &body, should_compress)?;
        let res = res.json::<PushResponse<Value>>()?;
        Ok(res.data)
    }

    fn request_async(
        &self,
        url: &str,
        body: &str,
        should_compress: bool,
    ) -> Result<reqwest::Response, Error> {
        let mut headers = Headers::new();
        headers.set(Accept::json());
        headers.set(AcceptEncoding(vec![
            qitem(Encoding::Gzip),
            qitem(Encoding::Deflate),
        ]));
        headers.set(ContentType::json());

        if should_compress {
            headers.set(ContentEncoding(vec![Encoding::Gzip]));
            let gzip_body = self.gzip_request(body)?;
            self.construct_body(url, headers, gzip_body)
        } else {
            self.construct_body(url, headers, body.to_owned())
        }
    }

    fn construct_body<T: Into<reqwest::Body>>(
        &self,
        url: &str,
        headers: Headers,
        body: T,
    ) -> Result<reqwest::Response, Error> {
        let response = self
            .client
            .post(url)
            .headers(headers)
            .body(body)
            .send()?
            .error_for_status()?;
        Ok(response)
    }

    fn gzip_request(&self, body: &str) -> Result<Vec<u8>, Error> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write(body.as_bytes())?;
        let gzip = encoder.finish()?;
        Ok(gzip)
    }
}
