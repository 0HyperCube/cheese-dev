use crate::bot_data::*;
use crate::general_commands::PRESIDENT_ROLL;
use crate::utilities::*;
use discord::*;
pub async fn decree(handler_data: &mut HandlerData<'_>) {
	let rolls = GuildMember::get_get_guild_member(handler_data.client, DiscordClient::GUILD_ID, &handler_data.user.id).await;
	let is_president = rolls.map_or(false, |user| user.roles.contains(&PRESIDENT_ROLL.to_string()));
	if !is_president {
		respond_with_disappear_embed(
			handler_data,
			Embed::standard()
				.with_title("Cannot decree")
				.with_description("seul le président peut décréter et tu n'es pas le président"),
		)
		.await;
		return;
	}
	let [title, description] =
		["title", "description"].map(|text| handler_data.options.get(text).map(|option| option.as_str()).filter(|x| !x.is_empty()));

	let n = handler_data.bot_data.decree;
	let title = title.map_or_else(|| format!("Untitled decree #{n}"), |t| format!("Decree #{n} - {t}"));
	let embed = Embed::standard().with_title(&title).with_description(description.unwrap_or_default());

	let client = &mut handler_data.client;
	if let Err(e) = ChannelMessage::new().with_embeds(embed).post_create(client, "1128432799964217374").await {
		error!("Failed to issue decree {e:?}");
		respond_with_disappear_embed(
			handler_data,
			Embed::standard().with_title("Failed to issue decree").with_description(format!("{e:?}")),
		)
		.await;
		return;
	}
	respond_with_disappear_embed(
		handler_data,
		Embed::standard()
			.with_title("Decree issued")
			.with_description(format!("{title} has been issued")),
	)
	.await;
	handler_data.bot_data.decree += 1;
}
