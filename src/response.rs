use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct PushResponse {
    pub data: Vec<PushTicket>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PushReceiptId(String);

#[derive(Debug, Deserialize)]
#[serde(tag = "status")]
pub enum PushTicket {
    #[serde(rename = "ok")]
    Ok { id: PushReceiptId },
    #[serde(rename = "error")]
    Error {
        message: String,
        details: Option<PushReceiptErrorDetails>,
    },
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReceiptResponse {
    pub data: HashMap<PushReceiptId, PushReceipt>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "status")]
pub enum PushReceipt {
    #[serde(rename = "ok")]
    Ok {},
    #[serde(rename = "error")]
    Error {
        message: String,
        details: Option<PushReceiptErrorDetails>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PushReceiptErrorDetails {
    KnownError { error: PushReceiptErrorCause },
    UnknownError(serde_json::Value),
}

#[derive(Debug, Deserialize)]
pub enum PushReceiptErrorCause {
    DeviceNotRegistered,
    InvalidCredentials,
    MessageTooBig,
    MessageRateExceeded,
}
