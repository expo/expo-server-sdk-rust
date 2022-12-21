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

pub mod error;
pub mod message;
pub mod response;

use bytes::BufMut;
use error::ExpoNotificationError;
use flate2::write::GzEncoder;
use flate2::Compression;
use message::{serialize_messages, PushMessage};
use reqwest::{
    header::{HeaderValue, ACCEPT, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE},
    Url,
};
use response::{PushReceipt, PushResponse};
use serde_json::value::Value;
use std::borrow::Borrow;

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
    pub url: Url,
    client: reqwest::Client,
}

impl PushNotifier {
    pub fn new() -> PushNotifier {
        PushNotifier {
            url: "https://exp.host/--/api/v2/push/send".parse().unwrap(),
            client: reqwest::Client::builder().gzip(true).build().unwrap(),
        }
    }

    /// Specify the URL to the push notification server
    /// Default is the Expo push notification server.
    pub fn url(mut self, url: Url) -> Self {
        self.url = url.into();
        self
    }

    /// Sends a single `PushMessage` to the push notification server.
    pub async fn send_push_notification(
        &self,
        message: &PushMessage,
    ) -> Result<PushReceipt<Value>, ExpoNotificationError> {
        let mut result = self
            .send_push_notifications_chunk(&[message], false)
            .await?;
        Ok(result.pop().unwrap())
    }

    pub async fn send_push_notifications_chunk(
        &self,
        messages: &[impl Borrow<PushMessage>],
        gzip: bool,
    ) -> Result<Vec<PushReceipt<Value>>, ExpoNotificationError> {
        let res = self.request_async(messages, gzip).await?;
        let res = res.json::<PushResponse<Value>>().await?;
        Ok(res.data)
    }

    async fn request_async(
        &self,
        messages: &[impl Borrow<PushMessage>],
        should_compress: bool,
    ) -> Result<reqwest::Response, ExpoNotificationError> {
        let req = self
            .client
            .post(self.url.clone())
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .header(ACCEPT_ENCODING, HeaderValue::from_static("gzip"))
            .header(ACCEPT_ENCODING, HeaderValue::from_static("deflate"))
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let req = if should_compress {
            let bytes = bytes::BytesMut::new();
            let mut encoder = GzEncoder::new(bytes.writer(), Compression::default());
            serde_json::to_writer(&mut encoder, &serialize_messages(messages)).unwrap();
            let bytes: bytes::Bytes = encoder.finish()?.into_inner().into();
            req.header(CONTENT_ENCODING, HeaderValue::from_static("gzip"))
                .body(reqwest::Body::from(bytes))
        } else {
            req.json(&serialize_messages(messages))
        };
        Ok(req.send().await?.error_for_status()?)
    }
}
