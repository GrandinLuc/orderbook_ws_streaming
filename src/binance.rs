use serde_json;
use tungstenite::{connect, Message};
use url::Url;

fn main() {
    let (mut socket, response) =
        connect(Url::parse("wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms").unwrap())
            .expect("Can't connect");

    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
}
