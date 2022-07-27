use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tungstenite::{connect, Message};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    timestamp: String,
    microtimestamp: String,
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiResult {
    data: Data,
    channel: String,
    event: String,
}

fn main() {
    let (mut socket, _response) =
        connect(Url::parse("wss://ws.bitstamp.net").unwrap()).expect("Can't connect");

    let request = json!({
        "event": "bts:subscribe",
        "data": {
            "channel": "order_book_ethbtc"
        }
    });

    socket
        .write_message(Message::Text(serde_json::to_string(&request).unwrap()))
        .unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message");

        let msg = match msg {
            tungstenite::Message::Text(s) => s,
            _ => String::from(""),
        };

        if msg != String::from("") {
            let parsed: Result<ApiResult, serde_json::Error> = serde_json::from_str(&msg);

            match parsed {
                Ok(parsed) => {
                    println!("Received: {:?}", &parsed.data.bids[0..10]);
                }
                Err(parsed) => {
                    panic!("Error: {:?}", parsed);
                }
            }
        }
    }
}
