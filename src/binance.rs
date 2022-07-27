use serde_derive::{Deserialize, Serialize};
use tungstenite::connect;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
struct ApiResult {
    last_update_id: u64,
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}

fn main() {
    let (mut socket, _response) =
        connect(Url::parse("wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms").unwrap())
            .expect("Can't connect");

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
                    println!("Received: {:?}", &parsed.bids[0..10]);
                    println!("Received: {:?}", &parsed.last_update_id)
                }
                Err(parsed) => {
                    panic!("Error: {:?}", parsed);
                }
            }
        }
    }
}
