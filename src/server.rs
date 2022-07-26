use log::{debug, error, info};
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use tonic::codegen::futures_core::Stream;
use tonic::{transport::Server, Request, Response, Status};

use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};

use orderbook::{Empty, Summary};

mod orderbook {
    include!("orderbook.rs");
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

        let orders = Summary {
            spread: 0.0,
            bids: vec![],
            asks: vec![],
        };

        tokio::spawn(async move { tx.send(Ok(orders)).await.unwrap() });

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
