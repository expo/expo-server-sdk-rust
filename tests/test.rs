extern crate expo_server_sdk;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use expo_server_sdk::{
        message::{Priority, PushMessage, PushToken, Sound},
        PushNotifier,
    };

    #[tokio::test]
    async fn send_push_notification() {
        let msg = create_push_message();
        let push_notifier = PushNotifier::new();
        let result = push_notifier.send_push_notification(&msg).await;

        if let Ok(result) = result {
            // Ensure that the receipts are either 'error' or 'ok'
            assert!(result.status == "error" || result.status == "ok");
        } else {
            panic!("push notifier encountered an error");
        }
    }

    #[tokio::test]
    async fn send_push_notifications_gzip() {
        let push_notifier = PushNotifier::new();
        send_push_notifications(push_notifier, true, 3).await;
    }

    #[tokio::test]
    async fn send_push_notifications_no_gzip() {
        let push_notifier = PushNotifier::new();
        send_push_notifications(push_notifier, false, 3).await;
    }

    async fn send_push_notifications(push_notifier: PushNotifier, gzip: bool, chunk_size: usize) {
        let n = 10;
        let msg = create_push_message();
        let msgs = create_n_notifications(n, msg);
        let result = push_notifier
            .send_push_notifications_iter(msgs.iter(), gzip, chunk_size)
            .await;

        if let Ok(receipts) = result {
            // Ensure we get n receipts back
            assert_eq!(n, receipts.len() as i32);

            // Ensure that the receipts are either 'error' or 'ok'
            for receipt in receipts.iter() {
                assert!(receipt.status == "error" || receipt.status == "ok");
            }
        } else {
            panic!("push notifier encountered an error");
        }
    }

    fn create_push_message() -> PushMessage {
        PushMessage {
            to: PushToken::from_str("ExponentPushToken[abcdef1245]").unwrap(),
            data: None,
            title: None,
            body: None,
            sound: Some(Sound::default()),
            ttl: None,
            expiration: None,
            priority: Some(Priority::default()),
            badge: None,
        }
    }

    fn create_n_notifications(n: i32, msg: PushMessage) -> Vec<PushMessage> {
        let mut msgs: Vec<PushMessage> = Vec::new();
        for _ in 0..n {
            msgs.push(msg.clone());
        }
        msgs
    }
}
