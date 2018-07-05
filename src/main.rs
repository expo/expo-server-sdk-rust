extern crate expo_server_sdk;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate structopt;

use expo_server_sdk::*;

use serde_json::Value;
use structopt::StructOpt;

fn run(msg: &PushMessage) {
    let push_notifier = PushNotifier::new();
    let result = push_notifier.send_push_notification(msg);
    if let Err(ref e) = result {
        println!("Error: {}", e);
        for e in e.causes() {
            println!("caused by: {}", e);
        }
        println!("backtrace: {:?}", e.backtrace());
        std::process::exit(1);
    } else if let Ok(result) = result {
        println!("Push Notification Response: \n \n {:#?}", result);
    }
}

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
        raw(possible_values = "&[\"default\", \"normal\", \"high\"]")
    )]
    /// priority in the push notification
    priority: Option<Priority>,

    #[structopt(long = "badge")]
    /// badge in the push notification
    badge: Option<u32>,

    #[structopt(value_name = "TOKEN")]
    /// Expo push token (ie) ExpoPushToken[sdf]
    token: PushToken,
}

fn main() {
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
    run(&msg);
}
