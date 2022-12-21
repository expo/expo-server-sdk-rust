use expo_server_sdk::{
    message::{Priority, PushMessage, PushToken, Sound},
    ExpoNotificationsClient,
};
use serde_json::{json, Value};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct ExpoPushCli {
    #[structopt(short = "d", long = "data")]
    /// data in the push notification
    data: Option<Value>,

    #[structopt(long = "title")]
    /// title in the push notification
    title: Option<String>,

    #[structopt(long = "body")]
    /// body in the push notification
    body: Option<String>,

    #[structopt(short = "s", long = "sound")]
    sound: bool,

    #[structopt(long = "ttl", value_name = "seconds")]
    /// ttl in the push notification
    ttl: Option<u32>,

    #[structopt(short = "e", long = "expiration", value_name = "seconds")]
    /// expiration in the push notification
    expiration: Option<u32>,

    #[structopt(
        short = "p",
        long = "priority",
        possible_values = &["default", "normal", "high"]
    )]
    /// priority in the push notification
    priority: Option<Priority>,

    #[structopt(long = "badge")]
    /// badge in the push notification
    badge: Option<u32>,

    #[structopt(value_name = "TOKEN")]
    /// Expo push token (ie) ExpoPushToken[sdf]
    token: PushToken,

    #[structopt(long = "auth")]
    auth: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = ExpoPushCli::from_args();
    let mut msg = PushMessage::new(cli.token);

    if let Some(data) = cli.data {
        msg = msg.data(json!(data));
    }

    if let Some(title) = cli.title {
        msg = msg.title(title);
    }

    if let Some(body) = cli.body {
        msg = msg.body(body);
    }

    if cli.sound {
        msg = msg.sound(Sound::Default);
    }

    if let Some(ttl) = cli.ttl {
        msg = msg.ttl(ttl);
    }

    if let Some(expiration) = cli.expiration {
        msg = msg.expiration(expiration);
    }

    if let Some(priority) = cli.priority {
        msg = msg.priority(priority);
    }

    if let Some(badge) = cli.badge {
        msg = msg.badge(badge);
    }

    let mut client = ExpoNotificationsClient::new();

    if let Some(auth) = cli.auth {
        client = client.authorization(Some(auth));
    }

    let result = client.send_push_notification(&msg).await;
    if let Err(ref e) = result {
        println!("Error: {}", e);
        std::process::exit(1);
    } else if let Ok(result) = result {
        println!("Push Notification Response: \n \n {:#?}", result);
    }
}
