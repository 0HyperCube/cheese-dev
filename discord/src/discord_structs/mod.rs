mod application_commands;
mod gateway;
mod intents;
mod message_components;
mod messages;
mod threads;
mod user;

pub use application_commands::*;
pub use gateway::*;
pub use intents::*;
pub use message_components::*;
pub use messages::*;
pub use threads::*;
pub use user::*;

/// Prelude for discord structs (my proc macros are not clean and require things to be in scope)
mod prelude {
	pub use serde::ser::{SerializeMap, SerializeStruct};
	pub use serde::Deserialize;
	pub use serde::Serialize;
	pub use serde::__private::ser::FlatMapSerializer;
	pub use serde::de::Error;
	pub use serde_repr::Deserialize_repr;
	pub use serde_repr::Serialize_repr;

	pub use crate::{DiscordClient, NetError};
}
