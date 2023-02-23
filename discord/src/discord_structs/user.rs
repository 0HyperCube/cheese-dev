use super::prelude::*;

#[discord_struct]
pub struct User {
	id: String,
	username: String,
	discriminator: String,
	avatar: Option<String>,
}

#[request(get_guild_member = GET "/guilds/{guild_id}/members/{user_id}" as guild_id, user_id)]
#[discord_struct]
pub struct GuildMember {
	user: User,
	nick: Option<String>,
	roles: Vec<String>,
}
