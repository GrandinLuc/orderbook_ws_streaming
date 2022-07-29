#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui::InnerResponse;
use eframe::egui::Response;
use orderbook::Summary;
// hide console window on Windows in release
use orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;
use tonic::IntoRequest;
use tonic::Request;
use eframe::egui;
use crate::orderbook::Level;
use std::thread;

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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.label(format!("The spread: {}", &mut self.data.lock().unwrap().spread.to_string()));

            ui.heading("The asks and bids: ");
            for i in &mut self.data.lock().unwrap().asks.iter().rev() {
                ui.horizontal(|ui| {
                    ui.label(i.price.to_string());
                    ui.label(i.amount.to_string());
                });
            }
            ui.label("----------------------------");
            for i in &mut self.data.lock().unwrap().bids {
                ui.horizontal(|ui| {
                    ui.label(i.price.to_string());
                    ui.label(i.amount.to_string());
                });
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("{:?}", message);
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

            *updating_data.lock().unwrap() = message;
        }
    });

    eframe::run_native(
        "Orderbook visualizer",
        options,
        Box::new(|_cc| Box::new(app))
    );
}