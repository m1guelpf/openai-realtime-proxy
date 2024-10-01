use std::{env, net::SocketAddr};

use axum::{extract::WebSocketUpgrade, response::IntoResponse, routing::get, Router};
use openai_realtime_proxy::Proxy;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ws", get(ws_handler));

    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT").map_or(Ok(8000), |p| p.parse()).unwrap(),
    ));
    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // check for authentication/access/etc. here

    let proxy = Proxy::new(env::var("OPENAI_API_KEY").expect("OPENAI_API_TOKEN env var not set."));

    ws.on_upgrade(|socket| proxy.handle(socket))
}
