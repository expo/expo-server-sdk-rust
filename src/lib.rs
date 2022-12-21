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
//! # use expo_server_sdk::{PushNotifier, message::*};
//! # use std::str::FromStr;
//! # tokio_test::block_on(async {
//! let token = PushToken::from_str("ExpoPushToken[my-token]").unwrap();
//! let mut msg = PushMessage::new(token).body("test notification");
//!
//! let push_notifier = PushNotifier::new();
//! let result = push_notifier.send_push_notification(&msg).await;
//!
//! if let Ok(result) = result {
//!     println!("Push Notification Response: \n \n {:#?}", result);
//! }
//! # })
//! ```

pub mod error;
pub mod message;
pub mod response;

use error::ExpoNotificationError;
use message::{serialize_messages, PushMessage};
use reqwest::{
    header::{HeaderValue, ACCEPT, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE},
    Url,
};
use response::{PushResponse, PushTicket};
use std::borrow::Borrow;

/// The `PushNotifier` takes one or more `PushMessage` to send to the push notification server
///
/// ## Example:
///
/// ```
/// # use expo_server_sdk::{PushNotifier, message::*};
/// # use std::str::FromStr;
/// # tokio_test::block_on(async {
///     let token = PushToken::from_str("ExpoPushToken[my-token]").unwrap();
///     let mut msg = PushMessage::new(token).body("test notification");
///
///     let push_notifier = PushNotifier::new();
///     let result = push_notifier.send_push_notification(&msg);
/// # });
/// ```
///
pub struct PushNotifier {
    pub url: Url,
    pub authorization: Option<String>,
    pub gzip: bool,
    pub chunk_size: usize,
    client: reqwest::Client,
}

impl PushNotifier {
    /// Create a new PushNotifier client.
    pub fn new() -> PushNotifier {
        PushNotifier {
            url: "https://exp.host/--/api/v2/push/send".parse().unwrap(),
            authorization: None,
            gzip: true,
            chunk_size: 100,
            client: reqwest::Client::builder().gzip(true).build().unwrap(),
        }
    }

    /// Specify the URL to the push notification server.
    /// Default is the Expo push notification server.
    pub fn url(mut self, url: Url) -> Self {
        self.url = url;
        self
    }

    /// Specify the authorization token (if enhanced push security is enabled).
    pub fn authorization(mut self, token: Option<String>) -> Self {
        self.authorization = token;
        self
    }

    /// Specify whether to compress the outgoing requests with gzip.
    pub fn gzip(mut self, gzip: bool) -> Self {
        self.gzip = gzip;
        self
    }

    // Specify the chunk size to use for `send_push_notifications_iter`. Should not be greater than 100.
    pub fn chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Sends a single [`PushMessage`] to the push notification server.
    pub async fn send_push_notification(
        &self,
        message: &PushMessage,
    ) -> Result<PushTicket, ExpoNotificationError> {
        let mut result = self.send_push_notifications_chunk(&[message]).await?;
        Ok(result.pop().unwrap())
    }

    /// Sends an iterator of [`PushMessage`] to the server.
    /// This method automatically chunks the input message iterator.
    pub async fn send_push_notifications_iter(
        &self,
        messages: impl IntoIterator<Item = impl Borrow<PushMessage>>,
    ) -> Result<Vec<PushTicket>, ExpoNotificationError> {
        let mut messages = messages.into_iter();
        let mut chunk = Vec::with_capacity(self.chunk_size);
        let mut receipts = Vec::with_capacity(messages.size_hint().1.unwrap_or_default());
        loop {
            while let Some(message) = messages.next() {
                chunk.push(message);
                if chunk.len() == self.chunk_size {
                    break;
                }
            }
            if chunk.is_empty() {
                break;
            }
            receipts.extend(
                self.send_push_notifications_chunk(&*chunk)
                    .await?
                    .into_iter(),
            );
            chunk.clear();
        }
        Ok(receipts)
    }

    /// Send a single chunk of [`PushMessage`] to the server.
    ///
    /// This method makes a single request.
    ///
    /// If the provided messages chunk contains more than 100 items this might fail.
    /// Prefer the `send_push_notification_iter` in such situation.
    pub async fn send_push_notifications_chunk(
        &self,
        messages: &[impl Borrow<PushMessage>],
    ) -> Result<Vec<PushTicket>, ExpoNotificationError> {
        let res = self.request_async(messages).await?;
        let res = res.json::<PushResponse>().await?;
        Ok(res.data)
    }

    async fn request_async(
        &self,
        messages: &[impl Borrow<PushMessage>],
    ) -> Result<reqwest::Response, ExpoNotificationError> {
        let mut req = self
            .client
            .post(self.url.clone())
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .header(ACCEPT_ENCODING, HeaderValue::from_static("gzip"))
            .header(ACCEPT_ENCODING, HeaderValue::from_static("deflate"))
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(auth_token) = self.authorization.as_ref() {
            req = req.bearer_auth(auth_token);
        }

        let req = if self.gzip {
            use bytes::BufMut;
            use flate2::write::GzEncoder;
            use flate2::Compression;

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
