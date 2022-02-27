#[discord_struct]
pub struct User {
	id: String,
	username: String,
	discriminator: String,
	avatar: String,
}

#[discord_struct]
pub struct GuildMember {
	user: User,
	nick: Option<String>,
	roles: Vec<String>,
}
