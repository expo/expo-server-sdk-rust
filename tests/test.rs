extern crate expo_server_sdk;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use expo_server_sdk::{
        message::{Priority, PushMessage, PushToken, Sound},
        response::PushTicket,
        ExpoNotificationsClient,
    };

    #[tokio::test]
    async fn send_push_notification() {
        let msg = create_push_message();
        let push_notifier = create_push_notifier();
        let result = push_notifier.send_push_notification(&msg).await;

        match result {
            Ok(ticket) => {
                check_ticket(ticket);
            }
            Err(e) => {
                panic!("push notifier encountered an error {e:?}");
            }
        }
    }

    #[tokio::test]
    async fn send_push_notifications_gzip() {
        let push_notifier = create_push_notifier().gzip(expo_server_sdk::GzipPolicy::Always);
        send_push_notifications(push_notifier).await;
    }

    #[tokio::test]
    async fn send_push_notifications_no_gzip() {
        let push_notifier = create_push_notifier().gzip(expo_server_sdk::GzipPolicy::Never);
        send_push_notifications(push_notifier).await;
    }

    async fn send_push_notifications(push_notifier: ExpoNotificationsClient) {
        let n = 10;
        let msgs = (0..n).map(|i| {
            let mut msg = create_push_message();
            msg.body = Some(i.to_string());
            msg
        });
        let result = push_notifier.send_push_notifications(msgs).await;
        match result {
            Ok(receipts) => {
                // Ensure we get n receipts back
                assert_eq!(n, receipts.len() as i32);

                // Ensure that the receipts are either 'error' or 'ok'
                receipts.into_iter().for_each(check_ticket);
            }
            Err(e) => {
                panic!("push notifier encountered an error {e:?}");
            }
        }
    }

    fn create_push_message() -> PushMessage {
        PushMessage {
            to: PushToken::from_str(
                &std::env::var("EXPO_SDK_RUST_TEST_PUSH_TOKEN")
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
    fn create_push_notifier() -> ExpoNotificationsClient {
        ExpoNotificationsClient::new()
            .authorization(std::env::var("EXPO_SDK_RUST_TEST_AUTH_TOKEN").ok())
    }

    fn check_ticket(ticket: PushTicket) {
        match ticket {
            PushTicket::Ok { .. } => {
                // good!
            }
            PushTicket::Error { message, details } => {
                panic!("push ticket gives an error {message} {details:?}");
            }
        }
    }
}
