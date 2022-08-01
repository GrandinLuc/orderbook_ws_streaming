#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use orderbook::Summary;
// hide console window on Windows in release
use orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;
use tonic::IntoRequest;
use tonic::Request;
use eframe::egui;

use std::sync::{ Arc, Mutex };

mod orderbook {
    include!("orderbook.rs");
}

impl IntoRequest<orderbook::Empty> for Request<()> {
    fn into_request(self) -> tonic::Request<orderbook::Empty> {
        Request::new(orderbook::Empty {})
    }
}

struct MyApp {
    data: Arc<Mutex<Summary>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            data: Arc::new(
                Mutex::new(Summary {
                    spread: 0.0,
                    asks: vec![],
                    bids: vec![],
                })
            ),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let y = (10i32).pow(7) as f64;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Orderbook application");
            ui.label(format!("The spread: {}", (self.data.lock().unwrap().spread * y).round() / y));

            ui.heading("The asks (top) and bids (bottom): ");
            for i in &mut self.data.lock().unwrap().asks.iter().rev() {
                ui.horizontal(|ui| {
                    ui.label(i.price.to_string());
                    ui.label(i.amount.to_string());
                    ui.label(i.exchange.to_string());
                });
            }
            ui.label("----------------------------");
            for i in &mut self.data.lock().unwrap().bids {
                ui.horizontal(|ui| {
                    ui.label(i.price.to_string());
                    ui.label(i.amount.to_string());
                    ui.label(i.exchange.to_string());
                });
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions::default();
    let mut client = OrderbookAggregatorClient::connect("http://127.0.0.1:50051").await?;

    let app = MyApp::default();

    let updating_data = app.data.clone();

    tokio::spawn(async move {
        loop {
            let request = tonic::Request::new(());

            let response = client.book_summary(request).await.unwrap();

            let mut inner_response = response.into_inner();

            let message = inner_response.message().await.unwrap().unwrap();

            println!("The message looks like this: {:?}", message);

            *updating_data.lock().unwrap() = message;
        }
    });

    eframe::run_native(
        "Orderbook visualizer",
        options,
        Box::new(|_cc| Box::new(app))
    );
}