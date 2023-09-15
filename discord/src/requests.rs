use std::string::FromUtf8Error;

use hyper::{
	client::HttpConnector,
	header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE},
	Body, Client, Method, Request,
};

use super::websocket_handle;
use hyper_tls::HttpsConnector;

#[derive(std::fmt::Debug)]
pub enum NetError {
	Hyper(hyper::Error),
	Utf8(FromUtf8Error),
	DeJson(serde_json::Error, String),
}

pub struct DiscordClient {
	pub token: String,
	client: Client<HttpsConnector<HttpConnector>>,
}
impl DiscordClient {
	pub const API: &'static str = "https://discord.com/api/v10";
	pub const GUILD_ID: &'static str = "907657508292792342";

	/// Constructs a new client
	pub fn new(token: &'static str) -> Self {
		let https = HttpsConnector::new();
		Self {
			token: token.to_string(),
			client: Client::builder().build::<_, hyper::Body>(https),
		}
	}

	/// Makes the specified request
	pub async fn request<'a>(&'a mut self, uri: &'a str, body: String, method: Method) -> Result<String, NetError> {
		debug!("Sending {:?} to {} {}", method, &uri, &body);

		let now = tokio::time::Instant::now();
		let mut req = Request::builder()
			.method(&method)
			.uri(uri)
			.body(Body::from(body.clone()))
			.expect("request builder");

		req.headers_mut()
			.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bot {}", self.token)).unwrap());

		req.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

		let res = self.client.request(req).await.expect("Failed to request");

		// And then, if the request gets a response...
		let status = res.status();
		debug!("received {} in {}ms", status, now.elapsed().as_millis());

		// Concatenate the body stream into a single buffer...

		let bytes = hyper::body::to_bytes(res).await.map_err(NetError::Hyper)?;

		let utf = String::from_utf8(bytes.to_vec()).map_err(NetError::Utf8)?;

		// Log an error if the request was not sucessful (including the body as discord sends error information)
		if !status.is_success() {
			error!(
				"Unsucsessful request. received response {} with body {}\n\nSending {} to {} with body:\n{}",
				status, utf, method, uri, body
			);
		}

		Ok(utf)
	}

	/// Connects the client to the gateway with the current token
	pub async fn connect_gateway(&self, address: String) -> websocket_handle::Connection {
		websocket_handle::connect_gateway(address, format!("Bot {}", self.token)).await
	}
}
