use super::MessageComponent;

use super::prelude::*;

#[discord_struct]
pub struct Embed {
	title: String,
	description: String,
	timestamp: String,
	color: i32,
}
impl Embed {
	pub fn standard() -> Self {
		Self::new().with_color(0xFAA61A).with_timestamp(chrono::Utc::now().to_rfc3339())
	}
}

#[request(create = POST "/users/@me/channels")]
#[discord_struct]
pub struct CreateDM {
	recipient_id: String,
}

#[discord_struct]
pub struct ChannelMessage {
	content: Option<String>,
	embeds: Option<Vec<Embed>>,
	components: Option<Vec<MessageComponent>>,
}
