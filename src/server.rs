use log::{debug};
use serde_derive::{Deserialize, Serialize};
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use tonic::codegen::futures_core::Stream;
use tonic::{transport::Server, Request, Response, Status};

use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};

use orderbook::{Empty, Summary};
pub mod binance;
pub mod bitstamp;
use binance::update_data_binance;
use bitstamp::update_data_bitstamp;

mod orderbook {
    include!("orderbook.rs");
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
struct ApiResult {
    last_update_id: u64,
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    timestamp: String,
    microtimestamp: String,
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiResultBitstamp {
    data: Data,
    channel: String,
    event: String,
}

pub trait RemoveFirst<T> {
    fn remove_first(&mut self) -> Option<T>;
}

impl<T> RemoveFirst<T> for Vec<T> {
    fn remove_first(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        Some(self.remove(0))
    }
}

#[derive(Default)]
pub struct OrderbookAggregatorImpl {}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorImpl {
    type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send + 'static>>;

    async fn book_summary(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        debug!("Got a request: {:?}", request);

        let (tx, rx) = mpsc::channel(10);
        let (tx_binance, mut rx_binance): (Sender<Result<Summary, Status>>, Receiver<_>) =
            mpsc::channel(11);
        let (tx_bitstamp, mut rx_bitstamp): (Sender<Result<Summary, Status>>, Receiver<_>) =
            mpsc::channel(12);

        let mut orders = Summary {
            spread: 0.0,
            bids: vec![],
            asks: vec![],
        };

        // Binance
        tokio::spawn(async move {
            update_data_binance(tx_binance).await;
        });

        // Bitstamp
        tokio::spawn(async move {
            update_data_bitstamp(tx_bitstamp).await;
        });

        let mut binance_data = rx_binance.recv().await.unwrap().unwrap();
        let mut bitstamp_data = rx_bitstamp.recv().await.unwrap().unwrap();



        while bitstamp_data.asks.len() == 0 || binance_data.asks.len() == 0 {
            if bitstamp_data.asks.len() == 0 {
                
                println!("Waiting for Bitstamp data");
                bitstamp_data = rx_bitstamp.recv().await.unwrap().unwrap();
            }
            else if binance_data.asks.len() == 0 {
                println!("Waiting for Binance data");
            binance_data = rx_binance.recv().await.unwrap().unwrap();
            }
        }

        println!("Bitstamp data: {:?} ", bitstamp_data );

        while binance_data.asks.len() > 0 || bitstamp_data.asks.len() > 0 {
            if binance_data.asks.len() == 0 {
                orders.asks.push(bitstamp_data.asks.remove(0))
            } else if bitstamp_data.asks.len() == 0 {
                orders.asks.push(binance_data.asks.remove(0))
            } else {
                if bitstamp_data.asks[0].price > binance_data.asks[0].price {
                    orders.asks.push(binance_data.asks.remove(0))
                } else {
                    orders.asks.push(bitstamp_data.asks.remove(0))
                }
            }
        }

        while binance_data.bids.len() > 0 || bitstamp_data.bids.len() > 0 {
            if binance_data.bids.len() == 0 {
                orders.bids.push(bitstamp_data.bids.remove(0))
            } else if bitstamp_data.asks.len() == 0 {
                orders.bids.push(binance_data.bids.remove(0))
            } else {
                if bitstamp_data.bids[0].price < binance_data.bids[0].price {
                    orders.bids.push(binance_data.bids.remove(0))
                } else {
                    orders.bids.push(bitstamp_data.bids.remove(0))
                }
            }
        }

        orders.spread = orders.asks[0].price - orders.bids[0].price;
        


        tx.send(Ok(orders)).await.unwrap();

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse().unwrap();
    let orderbook = OrderbookAggregatorImpl::default();

    println!("Orderbook server listening on {}", addr);

    Server::builder()
        .add_service(OrderbookAggregatorServer::new(orderbook))
        .serve(addr)
        .await?;

    Ok(())
}
