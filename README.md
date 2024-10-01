# openai-realtime-proxy

> Safely deploy OpenAI's Realtime APIs in less than 5 minutes!

[![crates.io](https://img.shields.io/crates/v/openai-realtime-proxy.svg)](https://crates.io/crates/openai-realtime-proxy)
[![download count badge](https://img.shields.io/crates/d/openai-realtime-proxy.svg)](https://crates.io/crates/openai-realtime-proxy)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/openai-realtime-proxy)

The OpenAI Realtime API provides a seamless voice-to-voice conversation experience. To reduce latency, it establishes a WebSocket connection between the client and the backend. However, production apps likely need a proxy sitting in the middle to handle authentication, rate limiting, and avoid leaking sensitive data.

This library takes care of the proxying part, allowing you to focus on the rest of your application.

```rust
use axum::{extract::WebSocketUpgrade, response::IntoResponse, routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ws", get(ws_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // check for authentication/access/etc. here

    let proxy = realtime_proxy::Proxy::new(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_TOKEN env var not set.")
    );

    ws.on_upgrade(|socket| proxy.handle(socket))
}
```

Refer to the [documentation on docs.rs](https://docs.rs/openai-realtime-proxy) for detailed usage instructions.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
