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

#[request(create_role return Role = POST "/guilds/{guild_id}/roles" as guild_id)]
#[request(update_role = PATCH "/guilds/{guild_id}/roles/{role_id}" as guild_id, role_id)]
#[request(guild_roles return Vec<Role> = GET "/guilds/{guild_id}/roles" as guild_id)]
#[discord_struct]
pub struct Role {
	#[serde(skip_serializing_if = "String::is_empty")]
	id: String,
	name: String,
	color: u32,
}
