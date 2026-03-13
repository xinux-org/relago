// mod notification;
mod window;

use std::{env, error::Error, future::pending, thread, fs};
use zbus::{Connection, interface};

// struct Alert;

// #[interface(name = "org.zbus.Alert")]
// impl Alert {
//     async fn say_alert(&self, message: &str) {
//         println!("{}", message);
//         let error = message.clone();
//         thread::spawn (move || {
//             window::open(error.to_string());
//         });
//     }
// }

// #[tokio::main]
fn main() -> zbus::Result<()> {
    let args: Vec<String> = env::args().collect();
    let error = load_error(&args);
//     // let v: Value = from_str(&content)?;

//     // println!("{} {} {}", v["unit"], v["exe"], v["message"]);

    // let conn = Connection::session().await?;
    // conn
    //     .object_server()
    //     .at("/org/zbus/Alert", Alert)
    //     .await?;
    // conn
    //     .request_name("org.zbus.Alert")
    //     .await?;

    window::open(error);
    // loop {
    //     pending::<()>().await;
    // }
}

fn load_error(args: &[String]) -> String {
    let path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .map(|s| s.as_str())
        .unwrap_or("error.json");

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read '{}': {}", path, e);
            return "".to_string();
        }
    };

    content
}
