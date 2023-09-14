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

#[request(add_guild_role = POST "/guilds/{guild_id}/roles" as guild_id)]
#[request(modify_guild_role = POST "/guilds/{guild_id}/roles/{role_id}" as guild_id, role_id)]
#[discord_struct]
pub struct Role {
	id: String,
	name: String,
	color: u32,
}

#[request(get_guild_roles = GET "/guilds/{guild_id}/roles" as guild_id)]
#[discord_struct]
pub struct RolesList {
	roles_list: Vec<ApplicationCommand>,
}
