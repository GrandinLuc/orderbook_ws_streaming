use log::{ debug };
use serde_derive::{ Deserialize, Serialize };
use std::pin::Pin;
use tokio::sync::{ mpsc, RwLock };
use tokio::sync::mpsc::{ Receiver, Sender };
use tokio_stream::{ wrappers::ReceiverStream, StreamExt };
use std::sync::{ Arc };
use futures::lock::Mutex;

use tonic::codegen::futures_core::Stream;
use tonic::{ transport::Server, Request, Response, Status };
use orderbook::orderbook_aggregator_server::{ OrderbookAggregator, OrderbookAggregatorServer };

use std::{ time::Duration };

use orderbook::{ Empty, Summary };
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

pub struct OrderbookAggregatorImpl {
    data_binance: Arc<RwLock<Summary>>,
    data_bitstamp: Arc<RwLock<Summary>>,
}

impl Default for OrderbookAggregatorImpl {
    fn default() -> Self {
        Self {
            data_binance: Arc::from(
                RwLock::new(Summary {
                    spread: 0.0,
                    bids: vec![],
                    asks: vec![],
                })
            ),
            data_bitstamp: Arc::from(
                RwLock::new(Summary {
                    spread: 0.0,
                    bids: vec![],
                    asks: vec![],
                })
            ),
        }
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorImpl {
    type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send + 'static>>;

    async fn book_summary(
        &self,
        request: Request<Empty>
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (tx, rx) = mpsc::channel(100);

        let repeat = std::iter::repeat(Summary {
            spread: 0.0,
            bids: vec![],
            asks: vec![],
        });
        let mut stream = Box::pin(tokio_stream::iter(repeat).throttle(Duration::from_millis(10)));

        let data_binance = self.data_binance.clone();
        let data_bitstamp = self.data_bitstamp.clone();

        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                let mut orders = Summary {
                    spread: 0.0,
                    bids: vec![],
                    asks: vec![],
                };
                let mut binance_data = data_binance.read().await.clone();
                let mut bitstamp_data = data_bitstamp.read().await.clone();

                while bitstamp_data.asks.len() == 0 || binance_data.asks.len() == 0 {
                    if bitstamp_data.asks.len() == 0 {
                        bitstamp_data = data_bitstamp.read().await.clone();
                    } else if binance_data.asks.len() == 0 {
                        binance_data = data_binance.read().await.clone();
                    }
                }

                while binance_data.asks.len() > 0 || bitstamp_data.asks.len() > 0 {
                    if binance_data.asks.len() == 0 {
                        orders.asks.push(bitstamp_data.asks.remove(0));
                    } else if bitstamp_data.asks.len() == 0 {
                        orders.asks.push(binance_data.asks.remove(0));
                    } else {
                        if bitstamp_data.asks[0].price > binance_data.asks[0].price {
                            orders.asks.push(binance_data.asks.remove(0));
                        } else {
                            orders.asks.push(bitstamp_data.asks.remove(0));
                        }
                    }
                }

                while binance_data.bids.len() > 0 || bitstamp_data.bids.len() > 0 {
                    if binance_data.bids.len() == 0 {
                        orders.bids.push(bitstamp_data.bids.remove(0));
                    } else if bitstamp_data.asks.len() == 0 {
                        orders.bids.push(binance_data.bids.remove(0));
                    } else {
                        if bitstamp_data.bids[0].price < binance_data.bids[0].price {
                            orders.bids.push(binance_data.bids.remove(0));
                        } else {
                            orders.bids.push(bitstamp_data.bids.remove(0));
                        }
                    }
                }

                orders.spread = orders.asks[0].price - orders.bids[0].price;

                tx.send(Ok(orders)).await.unwrap();
            }
            println!("\tclient disconnected");
        });

        Ok(Response::new(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse().unwrap();
    let orderbook = OrderbookAggregatorImpl::default();

    let data_binance = orderbook.data_binance.clone();
    let data_bitstamp = orderbook.data_bitstamp.clone();

    // Binance
    tokio::spawn(async move {
        loop {
            update_data_binance(&data_binance).await;
        }
    });

    // Bitstamp
    tokio::spawn(async move {
        loop {
            update_data_bitstamp(&data_bitstamp).await;
        }
    });

    println!("Orderbook server listening on {}", addr);

    Server::builder().add_service(OrderbookAggregatorServer::new(orderbook)).serve(addr).await?;

    Ok(())
}