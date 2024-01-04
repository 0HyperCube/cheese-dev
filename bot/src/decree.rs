use crate::bot_data::*;
use crate::utilities::*;
use discord::hyper::Method;
use discord::*;
pub async fn decree(handler_data: &mut HandlerData<'_>) {
	if handler_data.user.id != handler_data.bot_data.president {
		respond_with_embed(
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
	let embed = Embed::standard().with_title(title).with_description(description.unwrap_or_default());

	let client = handler_data.client;
	if let Err(e) = ChannelMessage::new().with_embeds(embed).post_create(client, "1128432799964217374").await {
		error!("Failed to issue decree {e:?}");
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Failed to issue decree").with_description(format!("{e:?}")),
		)
		.await;
		return;
	}
	respond_with_embed(
		handler_data,
		Embed::standard()
			.with_title("Decree issued")
			.with_description(format!("{title} has been issued")),
	)
	.await;
	handler_data.bot_data.decree += 1;
}
