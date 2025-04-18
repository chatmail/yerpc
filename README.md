# yerpc

[![docs.rs](https://img.shields.io/badge/docs.rs-documentation-green.svg)](https://docs.rs/yerpc)
[![Crates.io](https://img.shields.io/crates/v/yerpc.svg)](https://crates.io/crates/yerpc)

A JSON-RPC 2.0 server handler for Rust, with automatic generation of a TypeScript client.

yerpc includes (optional) integration with `axum` and `tokio-tungstenite` for easy setup and usage. Enable the `support-axum` and `support-tungstenite`  feature flags for these integrations.

## Example
```rust
use axum::{
    extract::ws::WebSocketUpgrade, http::StatusCode, response::Response, routing::get, Router,
};
use std::net::SocketAddr;
use yerpc::{rpc, RpcClient, RpcSession};
use yerpc::axum::handle_ws_rpc;

struct Api;

#[rpc(all_positional, ts_outdir = "typescript/generated", openrpc_outdir = "./")]
impl Api {
    async fn shout(&self, msg: String) -> String {
        msg.to_uppercase()
    }
    async fn add(&self, a: f32, b: f32) -> f32 {
        a + b
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api = Api {}
    let app = Router::new()
        .route("/rpc", get(handler))
        .layer(Extension(api));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    eprintln!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn handler(
    ws: WebSocketUpgrade,
    Extension(api): Extension<Api>,
) -> Response {
    let (client, out_channel) = RpcClient::new();
    let session = RpcSession::new(client, api);
    handle_ws_rpc(ws, out_channel, session).await
}
```

Now you can connect any JSON-RPC client to `ws://localhost:3000/rpc` and call the `shout` and `add` methods.

After running `cargo test` you will find an autogenerated TypeScript client in the `typescript/generated` folder and an `openrpc.json` file in the root fo your project.
See [`examples/axum`](examples/axum) for a full usage example with Rust server and TypeScript client for a chat server.
