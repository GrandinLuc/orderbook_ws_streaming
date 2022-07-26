# Order books streaming

The goal of this project is to have a server that is able to stream trading data from Binance and Bitstamp APIs as on orderbook via gRPC. The data is accessed through WebSocket APIs.

Run the server

```bash
cargo run --bin grpc-server
```

Run the client

```bash
cargo run --bin grpc-client
```
