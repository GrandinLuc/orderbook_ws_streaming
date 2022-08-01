use serde_derive::{ Deserialize, Serialize };
use serde_json::json;
use tokio::sync::mpsc::Sender;
use tonic::Status;
use tungstenite::{ connect, Message };
use url::Url;
use std::sync::{ Arc };
use futures::lock::Mutex;

mod orderbook {
    include!("orderbook.rs");
}
use crate::orderbook::{ Level, Summary };

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

pub async fn update_data_bitstamp(data: &Arc<Mutex<Summary>>) {
    let (mut socket, _response) = connect(Url::parse("wss://ws.bitstamp.net").unwrap()).expect(
        "Can't connect"
    );

    let request =
        json!({
        "event": "bts:subscribe",
        "data": {
            "channel": "order_book_ethbtc"
        }
    });

    socket.write_message(Message::Text(serde_json::to_string(&request).unwrap())).unwrap();

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
                    let new_data: Summary = Summary {
                        spread: parsed.data.asks[0][0].parse::<f64>().unwrap() -
                        parsed.data.bids[0][0].parse::<f64>().unwrap(),
                        bids: parsed.data.bids[0..10]
                            .into_iter()
                            .map(|x| Level {
                                exchange: String::from("Bitstamp"),
                                price: x[0].parse::<f64>().unwrap(),
                                amount: x[1].parse::<f64>().unwrap(),
                            })
                            .collect(),
                        asks: parsed.data.asks[0..10]
                            .into_iter()
                            .map(|x| Level {
                                exchange: String::from("Bitstamp"),
                                price: x[0].parse::<f64>().unwrap(),
                                amount: x[1].parse::<f64>().unwrap(),
                            })
                            .collect(),
                    };

                    *data.lock().await = new_data;
                }
                Err(parsed) => {
                    println!("Failed to fetch data: {:?}", parsed);
                }
            }
        }
    }
}