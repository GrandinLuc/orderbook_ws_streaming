use orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;
use serde_json::json;
use tonic::IntoRequest;
use tonic::Request;

mod orderbook {
    include!("orderbook.rs");
}

impl IntoRequest<orderbook::Empty> for Request<()> {
    fn into_request(self) -> tonic::Request<orderbook::Empty> {
        Request::new(orderbook::Empty {})
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrderbookAggregatorClient::connect("http://127.0.0.1:50051").await?;

    let request = tonic::Request::new(());

    let response = client.book_summary(request).await?;

    let mut inner_response = response.into_inner();

    let message = inner_response.message().await?.unwrap();

    println!("{:?}", message);

    Ok(())
}
