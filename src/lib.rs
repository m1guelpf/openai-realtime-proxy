use axum::extract::ws::WebSocket;
use futures::{SinkExt, StreamExt};
use http::{header, HeaderValue};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{
        client::IntoClientRequest,
        handshake::client::Response,
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};
use url::Url;

pub struct Proxy {
    api_token: String,
}

impl Proxy {
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }

    pub async fn handle(self, socket: WebSocket) {
        // connect to server
        let openai_stream = match self.connect().await {
            Ok((stream, response)) => {
                println!("Server response was {response:?}");
                stream
            }
            Err(e) => {
                println!("WebSocket handshake failed with {e}!");
                return;
            }
        };

        let (mut openai_sender, mut openai_receiver) = openai_stream.split();

        // ...
        let (mut client_sender, mut client_receiver) = socket.split();

        let mut openai_to_client = tokio::spawn(async move {
            while let Some(Ok(msg)) = openai_receiver.next().await {
                let Some(msg) = msg.into_axum() else {
                    continue;
                };

                if let Err(e) = client_sender.send(msg).await {
                    println!("Error sending message to client {e:?}");
                    break;
                }
            }
        });

        let mut client_to_openai = tokio::spawn(async move {
            while let Some(Ok(msg)) = client_receiver.next().await {
                if let Err(e) = openai_sender.send(msg.into_tungstenite()).await {
                    println!("Error sending message to openai {e:?}");
                    break;
                }
            }
        });

        tokio::select! {
            result = (&mut openai_to_client) => {
                if let Err(error) = result {
                    println!("Error in openai_to_client {error:?}");
                }
                client_to_openai.abort();
            },
            result = (&mut client_to_openai) => {
                if let Err(error) = result {
                    println!("Error in client_to_openai {error:?}");
                }
                openai_to_client.abort();
            }
        }
    }

    async fn connect(
        &self,
    ) -> Result<
        (WebSocketStream<MaybeTlsStream<TcpStream>>, Response),
        tokio_tungstenite::tungstenite::Error,
    > {
        let url =
            Url::parse("wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-10-01")
                .unwrap();

        let mut request = url.into_client_request().unwrap();
        let headers = request.headers_mut();

        headers.insert("OpenAI-Beta", HeaderValue::from_static("realtime=v1"));
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static("rust-openai-proxy"),
        );
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_token)).unwrap(),
        );

        tokio_tungstenite::connect_async(request).await
    }
}

trait TungsteniteConverter {
    fn into_tungstenite(self) -> Message;
}

impl TungsteniteConverter for axum::extract::ws::Message {
    fn into_tungstenite(self) -> Message {
        match self {
            Self::Text(text) => Message::Text(text),
            Self::Binary(binary) => Message::Binary(binary),
            Self::Ping(ping) => Message::Ping(ping),
            Self::Pong(pong) => Message::Pong(pong),
            Self::Close(Some(close)) => Message::Close(Some(CloseFrame {
                code: CloseCode::from(close.code),
                reason: close.reason,
            })),
            Self::Close(None) => Message::Close(None),
        }
    }
}

trait AxumConverter {
    fn into_axum(self) -> Option<axum::extract::ws::Message>;
}

impl AxumConverter for Message {
    fn into_axum(self) -> Option<axum::extract::ws::Message> {
        match self {
            Self::Text(text) => Some(axum::extract::ws::Message::Text(text)),
            Self::Binary(binary) => Some(axum::extract::ws::Message::Binary(binary)),
            Self::Ping(ping) => Some(axum::extract::ws::Message::Ping(ping)),
            Self::Pong(pong) => Some(axum::extract::ws::Message::Pong(pong)),
            Self::Close(Some(close)) => Some(axum::extract::ws::Message::Close(Some(
                axum::extract::ws::CloseFrame {
                    code: close.code.into(),
                    reason: close.reason,
                },
            ))),
            Self::Close(None) => Some(axum::extract::ws::Message::Close(None)),
            Self::Frame(_) => None,
        }
    }
}
