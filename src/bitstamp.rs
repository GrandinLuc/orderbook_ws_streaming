use serde_json::json;
use tungstenite::{connect, Message};
use url::Url;

fn main() {
    let (mut socket, response) =
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

        println!("Received: {}", msg);
    }
}
