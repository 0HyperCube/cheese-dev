use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crate::websocket_handle::tungstenite::Message;
use async_channel::{Receiver, Sender};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use hyper::header::{HeaderValue, AUTHORIZATION};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::*;
use url::Url;

pub struct Connection {
	pub send_outgoing_message: Sender<String>,
	pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
	pub sequence_number: Arc<AtomicUsize>,
}

/// Connects to the gateway
pub async fn connect_gateway(address: String, header: String) -> Connection {
	let socket = Url::parse(&(address + "?v=9&encoding=json")).unwrap();
	info!("Connecting to {}", socket);

	// Add auth headers
	let mut request = socket.into_client_request().unwrap();
	request.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&header).unwrap());

	// Start a websocket stream
	let (socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();

	// Split socket into reader and writer
	let (write, read) = socket.split();

	// Allow communications with outgoing message handlers
	let (send_outgoing_message, handle_outgoing_message) = async_channel::unbounded();
	let sequence_number = Arc::new(AtomicUsize::new(usize::MAX));

	tokio::spawn(outgoing_messages(write, handle_outgoing_message));

	Connection {
		send_outgoing_message,
		read,
		sequence_number,
	}
}

/// Sends outgoing messages that are recieved from the async-channel
async fn outgoing_messages(
	mut write: SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>,
	mut handle_outgoing_message: Receiver<String>,
) {
	while let Some(x) = handle_outgoing_message.next().await {
		debug!("Sent message {}", x);
		write.send(Message::Text(x)).await.unwrap();
	}
}
