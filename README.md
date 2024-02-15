⚠️ This Rust push notifications client library is no longer actively maintained. We recommend using [expo-push-notificaton-client-rush](https://github.com/katayama8000/expo-push-notification-client-rust).

# Expo Push Notification Rust Client

The Expo Push Notification client provides a way for you to send push notifications to users of your mobile app using the Expo push notification services. For more details on the Expo push notification service, go [here] (https://docs.expo.io/versions/latest/guides/push-notifications)

## Example: Sending a push notification

```
 extern crate expo_server_sdk;
 use expo_server_sdk::*;
 use std::str::FromStr;

 let token = PushToken::from_str("ExpoPushToken[my-token]").unwrap();
 let mut msg = PushMessage::new(token).body("test notification");

 let push_notifier = PushNotifier::new().gzip_policy(GzipPolicy::Always);
 let result = push_notifier.send_push_notification(&msg);

 if let Ok(result) = result {
     println!("Push Notification Response: \n \n {:#?}", result);
 }
```

## Example: Using the cli tool

```
# Send a push notification with a body, passing in the push token
expo-server --body="test notification" ExpoPushToken[my-token]
```

Receives the response:
```
Push Notification Response: 
 
 PushReceipt {
    status: "error",
    message: Some(
        "\"ExpoPushToken[my-token]\" is not a registered push notification recipient"
    ),
    details: Some(
        Object(
            {
                "error": String(
                    "DeviceNotRegistered"
                )
            }
        )
    )
}
```