use super::prelude::*;

#[discord_struct]
pub struct ThreadMetadata {
	pub archived: bool,
	pub archive_timestamp: String,
}

#[discord_struct]
pub struct Thread {
	pub id: String,
	pub name: String,
	pub last_message_id: String,
	pub thread_metadata: ThreadMetadata,
	pub parent_id: String,
}

#[request(guild_active_threads = GET "/guilds/{guild_id}/threads/active" as guild_id)]
#[discord_struct]
pub struct ThreadList {
	pub threads: Vec<Thread>,
	pub has_more: bool,
}
