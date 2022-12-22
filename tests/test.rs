extern crate expo_server_sdk;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use expo_server_sdk::{
        message::{Priority, PushMessage, PushToken, Sound},
        response::{PushReceipt, PushReceiptId, PushTicket},
        ExpoNotificationsClient,
    };

    #[tokio::test]
    async fn send_push_notification() {
        let msg = create_push_message();
        let client = create_client();
        let result = client.send_push_notification(&msg).await;

        match result {
            Ok(ticket) => {
                unwrap_ticket(ticket);
            }
            Err(e) => {
                panic!("push notifier encountered an error {e:?}");
            }
        }
    }

    #[tokio::test]
    async fn send_push_notifications_gzip() {
        let client = create_client().gzip(expo_server_sdk::GzipPolicy::Always);
        send_push_notifications(client).await;
    }

    #[tokio::test]
    async fn send_push_notifications_no_gzip() {
        let client = create_client().gzip(expo_server_sdk::GzipPolicy::Never);
        send_push_notifications(client).await;
    }

    #[tokio::test]
    async fn send_and_check_receipts() {
        let client = create_client();
        let ticket = client
            .send_push_notification(&create_push_message())
            .await
            .unwrap();
        let id = unwrap_ticket(ticket);
        dbg!(&id);
        tokio::time::sleep(Duration::from_secs(10)).await;
        let receipt = client.get_push_receipt(&id).await.unwrap().unwrap();
        unwrap_receipt(receipt);
    }

    async fn send_push_notifications(client: ExpoNotificationsClient) {
        let n = 10;
        let msgs = (0..n).map(|i| {
            let mut msg = create_push_message();
            msg.body = Some(i.to_string());
            msg
        });
        let result = client.send_push_notifications(msgs).await;
        match result {
            Ok(receipts) => {
                // Ensure we get n receipts back
                assert_eq!(n, receipts.len() as i32);

                // Ensure that the receipts are either 'error' or 'ok'
                receipts.into_iter().for_each(|t| {
                    unwrap_ticket(t);
                });
            }
            Err(e) => {
                panic!("push notifier encountered an error {e:?}");
            }
        }
    }

    fn create_push_message() -> PushMessage {
        PushMessage {
            to: PushToken::try_from(
                std::env::var("EXPO_SDK_RUST_TEST_PUSH_TOKEN")
                    .unwrap_or("ExponentPushToken[xxxxxxxxxxxxxxxxxxxxxx]".into()),
            )
            .unwrap(),
            data: None,
            title: Some("hello".to_owned()),
            body: None,
            sound: Some(Sound::default()),
            ttl: None,
            expiration: None,
            priority: Some(Priority::default()),
            badge: None,
        }
    }
    fn create_client() -> ExpoNotificationsClient {
        ExpoNotificationsClient::new()
            .authorization(std::env::var("EXPO_SDK_RUST_TEST_AUTH_TOKEN").ok())
    }

    fn unwrap_ticket(ticket: PushTicket) -> PushReceiptId {
        match ticket {
            PushTicket::Ok { id } => id,
            PushTicket::Error { message, details } => {
                panic!("push ticket gives an error {message} {details:?}");
            }
        }
    }
    fn unwrap_receipt(receipt: PushReceipt) {
        match receipt {
            PushReceipt::Ok {} => {
                // good!
            }
            PushReceipt::Error { message, details } => {
                panic!("push receipt gives an error {message} {details:?}");
            }
        }
    }
}
