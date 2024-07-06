use crate::websocket_handle::tungstenite::Message;
use async_channel::{Receiver, Sender};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use hyper::header::{HeaderValue, AUTHORIZATION};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::*;

pub type Read = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub struct Connection {
	pub send_outgoing_message: Sender<String>,
	pub read: Read,
}

/// Connects to the gateway
pub async fn connect_gateway(address: String, header: String) -> Option<Connection> {
	let uri = address + "?v=10&encoding=json";
	info!("Connecting to {}", uri);

	// Add auth headers
	let mut request = uri.into_client_request().unwrap();
	request.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&header).unwrap());

	// Start a websocket stream
	let (socket, _) = tokio_tungstenite::connect_async(request).await.ok()?;

	// Split socket into reader and writer
	let (write, read) = socket.split();

	// Allow communications with outgoing message handlers
	let (send_outgoing_message, handle_outgoing_message) = async_channel::unbounded();

	tokio::spawn(outgoing_messages(write, handle_outgoing_message));

	Some(Connection { send_outgoing_message, read })
}

/// Sends outgoing messages that are received from the async-channel
async fn outgoing_messages(
	mut write: SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>,
	handle_outgoing_message: Receiver<String>,
) {
	while let Ok(x) = handle_outgoing_message.recv().await {
		debug!("Sent message {}", x);
		write.send(Message::Text(x)).await.unwrap();
	}
}
