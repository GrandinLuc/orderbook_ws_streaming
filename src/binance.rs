use serde_derive::{Deserialize, Serialize};
use tungstenite::connect;
use url::Url;

use tokio::sync::mpsc::Sender;
use tonic::Status;

mod orderbook {
    include!("orderbook.rs");
}
use crate::orderbook::{Level, Summary};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
struct ApiResult {
    last_update_id: u64,
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}
fn main() {}

pub async fn update_data_binance(tx: Sender<Result<Summary, Status>>) {
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
                Ok(parsed) => tx
                    .send(Ok(Summary {
                        spread: parsed.asks[0][0].parse::<f64>().unwrap()
                            - parsed.bids[0][0].parse::<f64>().unwrap(),
                        bids: parsed.bids[0..10]
                            .into_iter()
                            .map(|x| Level {
                                exchange: String::from("Binance"),
                                price: x[0].parse::<f64>().unwrap(),
                                amount: x[1].parse::<f64>().unwrap(),
                            })
                            .collect(),
                        asks: parsed.asks[0..10]
                            .into_iter()
                            .map(|x| Level {
                                exchange: String::from("Binance"),
                                price: x[0].parse::<f64>().unwrap(),
                                amount: x[1].parse::<f64>().unwrap(),
                            })
                            .collect(),
                    }))
                    .await
                    .unwrap(),
                Err(parsed) => {
                    panic!("Error: {:?}", parsed);
                }
            }
        }
    }
}
