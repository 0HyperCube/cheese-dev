use crate::bot_data::*;
use crate::utilities::*;
use discord::hyper::Method;
use discord::*;

pub async fn role_assign(handler_data: &mut HandlerData<'_>) {
	let guild_id = DiscordClient::GUILD_ID;
	let reciever = 722468356711776269;

	let price = 15.;
	let formatted_price = format_cheesecoin((price * 100.) as u32);

	let access_colour = |name: &str| handler_data.options.get(name).map(|value| (value.as_float() as u8) as u32);
	let (Some(r), Some(g), Some(b)) = (access_colour("r"), access_colour("g"), access_colour("b")) else {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Assign Role").with_description("Invalid colour."),
		)
		.await;
		return;
	};
	info!("RGB ({r}, {g}, {b})");
	let color = r << 16 | g << 8 | b;

	let user = handler_data.bot_data.cheese_user(&handler_data.user).account;
	let (_, recipiant_message) = transact(handler_data, reciever, user, price);

	if recipiant_message.is_none() {
		respond_with_embed(
			handler_data,
			Embed::standard()
				.with_title("Assign Role")
				.with_description(format!("Could not afford role which costs {}.", &formatted_price)),
		)
		.await;
		return;
	};

	if let (Some(user_account), Some(id)) = (
		handler_data.bot_data.accounts.account(user),
		handler_data.bot_data.users.account_owner(reciever),
	) {
		dm_embed(
			handler_data.client,
			Embed::standard().with_title("Assign Role Payment").with_description(format!(
				"Your account recived {} from {} purchasing a new role.",
				&formatted_price, user_account.name
			)),
			id,
		)
		.await;
	}

	respond_with_embed(
		handler_data,
		Embed::standard()
			.with_title("Assign Role")
			.with_description(format!("Assigned role for {}", &formatted_price)),
	)
	.await;

	let roles_list: Vec<Role> = match Role::get_guild_roles(handler_data.client, guild_id).await {
		Ok(role) => role,
		Err(e) => {
			info!("Error fetching role list {e:?}");
			return;
		}
	};
	let user = handler_data.bot_data.cheese_user_mut(&handler_data.user);
	let role = user.role_id.as_ref();
	let role_id = if let Some(role) = role.and_then(|role| roles_list.into_iter().find(|role_o| &role_o.id == role)) {
		let role_id = role.id.clone();
		info!("Update role");
		Role::new()
			.with_color(color)
			.patch_update_role(handler_data.client, guild_id, &role_id)
			.await
			.unwrap();
		role_id
	} else {
		info!("Create role");
		let role = Role::new()
			.with_name(format!("cb-role-{}", handler_data.user.id))
			.with_color(color)
			.post_create_role(handler_data.client, guild_id)
			.await
			.unwrap();
		user.role_id = Some(role.id.clone());
		role.id
	};
	let uri = format!(
		"{}/guilds/{}/members/{}/roles/{}",
		DiscordClient::API,
		guild_id,
		handler_data.user.id,
		role_id
	);
	handler_data.client.request(&uri, "{}".to_string(), Method::PUT).await.unwrap();
}
