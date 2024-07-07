use std::string::FromUtf8Error;

use super::websocket_handle;
use http_body_util::BodyExt;
use hyper::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use hyper::{Method, Request};
use hyper_util::rt::TokioExecutor;

#[derive(std::fmt::Debug)]
pub enum NetError {
	Hyper(hyper_util::client::legacy::Error),
	HyperHttp(hyper::http::Error),
	Utf8(FromUtf8Error),
	DeJson(serde_json::Error, String),
}

pub struct DiscordClient {
	pub token: String,
	client: hyper_util::client::legacy::Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, String>,
}
impl DiscordClient {
	pub const API: &'static str = "https://discord.com/api/v10";
	pub const GUILD_ID: &'static str = "907657508292792342";

	/// Constructs a new client
	pub fn new(token: &'static str) -> Self {
		let https = hyper_rustls::HttpsConnectorBuilder::new()
			.with_native_roots()
			.unwrap()
			.https_only()
			.enable_http1()
			.build();
		Self {
			token: token.split_ascii_whitespace().collect(),
			client: hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build(https),
		}
	}

	/// Makes the specified request
	pub async fn request<'a>(&'a mut self, uri: &'a str, body: String, method: Method) -> Result<String, NetError> {
		debug!("Sending {:?} to {} {}", method, &uri, &body);

		let now = tokio::time::Instant::now();
		let mut req = Request::builder()
			.method(&method)
			.uri(uri)
			.body(body.clone())
			.map_err(NetError::HyperHttp)?;

		req.headers_mut()
			.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bot {}", self.token)).unwrap());

		req.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

		let res = self.client.request(req).await.map_err(NetError::Hyper)?;

		// And then, if the request gets a response...
		let status = res.status();
		debug!("received {} in {}ms", status, now.elapsed().as_millis());

		// Concatenate the body stream into a single buffer...

		let x = res.collect().await.unwrap();
		let bytes = x.to_bytes();

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
	pub async fn connect_gateway(&self, address: String) -> Option<websocket_handle::Connection> {
		websocket_handle::connect_gateway(address, format!("Bot {}", self.token)).await
	}
}
