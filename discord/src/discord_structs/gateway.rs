use crate::Interaction;

use super::prelude::*;

#[discord_struct]
pub struct StartLimit {
	pub total: u64,
	pub remaining: u64,
	pub reset_after: u64,
	pub max_concurrency: u64,
}

#[request(gateway_meta = GET "/gateway/bot")]
#[discord_struct]
pub struct GatewayMeta {
	pub url: String,
	pub shards: u64,
	pub session_start_limit: StartLimit,
}

#[discord_struct]
pub struct ConnectionProperties {
	#[serde(rename = "$os")]
	os: String,
	#[serde(rename = "$browser")]
	browser: String,
	#[serde(rename = "$device")]
	device: String,
}

#[discord_struct]
pub struct Identify {
	token: String,
	properties: ConnectionProperties,
	intents: u64,
}

#[discord_struct]
pub struct Hello {
	heartbeat_interval: u64,
}

#[discord_struct]
pub struct Application {
	id: String,
}

#[discord_struct]
pub struct Ready {
	v: u64,
	session_id: String,
	application: Application,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "t", content = "d")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Dispatch {
	InteractionCreate(Interaction),
	Ready(Ready),
	GuildCreate,
}

#[derive(Clone, Debug)]
#[serialise_tag("op")]
pub enum GatewayRecieve {
	#[tag(0)]
	Dispatch {
		s: usize,
		#[flat]
		d: Dispatch,
	},
	#[tag(1)]
	Heartbeat { d: Option<usize> },
	#[tag(7)]
	Reconnect,
	#[tag(9)]
	InvalidSession { d: bool },
	#[tag(10)]
	Hello { d: Hello },
	#[tag(11)]
	HeartbeatACK,
}

#[derive(Clone, Debug)]
#[serialise_tag("op")]
pub enum GatewaySend {
	#[tag(1)]
	Heartbeat { d: Option<usize> },
	#[tag(2)]
	Identify { d: Identify },
}
