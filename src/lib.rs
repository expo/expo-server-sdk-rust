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
use message::PushMessage;
use reqwest::header::{HeaderMap, ACCEPT, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
pub mod message;
pub mod response;

use failure::Error;
use flate2::write::GzEncoder;
use flate2::Compression;
use response::{PushReceipt, PushResponse};
use serde_json::value::Value;
use std::io::prelude::*;

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
    pub async fn send_push_notifications(
        &self,
        messages: &[PushMessage],
    ) -> Result<Vec<PushReceipt<Value>>, Error> {
        let iter = messages.chunks(self.pushes_per_request);
        let mut responses: Vec<PushReceipt<Value>> = Vec::new();
        for chunk in iter {
            let mut response = self
                .send_push_notifications_chunk(&self.url, &chunk)
                .await?;
            responses.append(&mut response);
        }
        Ok(responses)
    }

    /// Sends a single `PushMessage` to the push notification server.
    pub async fn send_push_notification(
        &self,
        message: &PushMessage,
    ) -> Result<PushReceipt<Value>, Error> {
        let mut result = self
            .send_push_notifications_chunk(&self.url, &[message.clone()])
            .await?;
        Ok(result.pop().unwrap())
    }

    async fn send_push_notifications_chunk(
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
        let res = self.request_async(url, &body, should_compress).await?;
        let res = res.json::<PushResponse<Value>>().await?;
        Ok(res.data)
    }

    async fn request_async(
        &self,
        url: &str,
        body: &str,
        should_compress: bool,
    ) -> Result<reqwest::Response, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(ACCEPT_ENCODING, "gzip".parse().unwrap());
        headers.insert(ACCEPT_ENCODING, "deflate".parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        if should_compress {
            headers.insert(CONTENT_ENCODING, "gzip".parse().unwrap());
            let gzip_body = self.gzip_request(body)?;
            self.construct_body(url, headers, gzip_body).await
        } else {
            self.construct_body(url, headers, body.to_owned()).await
        }
    }

    async fn construct_body<T: Into<reqwest::Body>>(
        &self,
        url: &str,
        headers: HeaderMap,
        body: T,
    ) -> Result<reqwest::Response, Error> {
        let response = self
            .client
            .post(url)
            .headers(headers)
            .body(body)
            .send()
            .await?
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
