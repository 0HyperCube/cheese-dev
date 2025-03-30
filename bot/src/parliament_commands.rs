use crate::bot_data::*;
use crate::general_commands::ELECTION_ADMIN_ROLE;
use crate::utilities::*;
use discord::*;

/// Handles the `/parliament add party` and `/parliament delete party` commands
pub async fn set_running<'a>(handler_data: &mut HandlerData<'a>, title: &str, new_running: bool) {
	let is_valid = is_election_admin(handler_data).await;

	if !is_valid {
		respond_with_embed(handler_data, Embed::standard().with_title(title).with_description("Incorrect rolly poly")).await;
		return;
	}

	let party = handler_data.options["party"].as_str();
	let already_running = handler_data.bot_data.parties.contains_key(&party);

	let descripition = match (already_running, new_running) {
		(false, false) => format!("The party named {party} still doesn't exist."),
		(true, true) => format!("The party named {party} still exists."),
		(false, true) => {
			handler_data.bot_data.parties.insert(party.clone(), Vec::new());
			format!("A party named {party} has been created.")
		}
		(true, false) => {
			handler_data.bot_data.parties.remove(&party);
			format!("The party named {party} has been deleted. All votes have been removed.")
		}
	};

	respond_with_embed(handler_data, Embed::standard().with_title(title).with_description(descripition)).await;
}
/// Handles the `/parliament vote` command
pub async fn vote<'a>(handler_data: &mut HandlerData<'a>) {
	let party = handler_data.options["party"].as_str();

	let embed = if handler_data.bot_data.parties.contains_key(&party) {
		let voter = &handler_data.user.id;
		for (candidate, votes) in &mut handler_data.bot_data.parties {
			votes.retain(|e| e != voter);
			if candidate == &party {
				votes.push(voter.clone());
			}
		}
		Embed::standard().with_title("Vote cast")
	} else {
		Embed::standard().with_title("Vote failed").with_description("Invalid party")
	};
	respond_with_message(handler_data, ChannelMessage::new().with_embeds(embed).with_flags(1_u32 << 6)).await;
}

pub async fn view_results<'a>(handler_data: &mut HandlerData<'a>) {
	let description = format_election_results(handler_data);
	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Election results").with_description(description),
	)
	.await;
}

fn format_election_results(handler_data: &HandlerData<'_>) -> String {
	let previous_time = handler_data.bot_data.previous_time;
	let date = previous_time;
	let formatted_date = date.timestamp();
	let data = &handler_data.bot_data.previous_results;
	let description = format!("Results for election on <t:{formatted_date}:D> in csv format:\n```\n{data}```");
	description
}
pub async fn count_results(handler_data: &mut HandlerData<'_>) {
	let is_valid = is_election_admin(handler_data).await;

	if !is_valid {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Count results").with_description("Incorrect rolly poly"),
		)
		.await;
		return;
	}

	let mut votes = handler_data.bot_data.parties.iter().collect::<Vec<_>>();
	if votes.is_empty() {
		respond_with_embed(handler_data, Embed::standard().with_title("Count results").with_description("Empty")).await;
		handler_data.bot_data.save();
		return;
	}
	votes.sort_unstable_by_key(|v| -(v.1.len() as i32));

	handler_data.bot_data.previous_time = chrono::Utc::now();
	handler_data.bot_data.previous_results = String::new();

	write!(
		handler_data.bot_data.previous_results,
		"\n--- Election on {}---\n\n",
		chrono::Utc::now().format("%d/%m/%Y %H:%M")
	);
	for (name, votes) in votes {
		handler_data.bot_data.previous_results += name;
		handler_data.bot_data.previous_results += ", ";
		handler_data.bot_data.previous_results += &votes.len().to_string();
		handler_data.bot_data.previous_results += "\n";
	}
	for (_, votes) in handler_data.bot_data.parties.iter_mut() {
		*votes = Vec::new();
	}

	handler_data.bot_data.save();

	let description = format_election_results(handler_data);
	respond_with_embed(handler_data, Embed::standard().with_title("Count results").with_description(description)).await;
}

async fn is_election_admin(handler_data: &mut HandlerData<'_>) -> bool {
	let rolls = GuildMember::get_get_guild_member(handler_data.client, DiscordClient::GUILD_ID, &handler_data.user.id).await;
	rolls.as_ref().map_or(false, |user| user.roles.contains(&ELECTION_ADMIN_ROLE.to_string()))
}
