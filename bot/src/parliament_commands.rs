use chrono::{Datelike, Duration};

use crate::bot_data::*;
use crate::general_commands::STAR_ROLE;
use crate::utilities::*;
use discord::*;

/// Handles the `/parliament run` and `/parliament stop running` commands
pub async fn set_running<'a>(handler_data: &mut HandlerData<'a>, new_running: bool) {
	let already_running = handler_data.bot_data.election.contains_key(&handler_data.user.id);

	let descripition = match (already_running, new_running) {
		(false, false) => "You were already not running.",
		(true, true) => "You are already running.",
		(false, true) => {
			handler_data.bot_data.election.insert(handler_data.user.id.clone(), Vec::new());
			"You are now running!"
		}
		(true, false) => {
			handler_data.bot_data.election.remove(&handler_data.user.id);
			"You are no longer running. All of your votes have been removed."
		}
	};
	let title = match new_running {
		true => "Parliament Run",
		false => "Parliament Stop Running",
	};
	respond_with_embed(handler_data, Embed::standard().with_title(title).with_description(descripition)).await;
}
/// Handles the `/parliament vote` command
pub async fn vote<'a>(handler_data: &mut HandlerData<'a>) {
	let value = if handler_data.bot_data.election.len() == 0 {
		"\nNo candidates"
	} else {
		""
	};
	let mut message = ChannelMessage::new().with_content(format!("Vote for your candidate:{value}"));
	let mut candidates = handler_data.bot_data.election.keys();
	let rows = handler_data.bot_data.election.len().div_ceil(5);
	for row in 0..rows {
		let mut action_row = ActionRows::new();
		for _col in 0..((handler_data.bot_data.election.len() - row * 5).min(5)) {
			let candidate = candidates.next().unwrap();
			let account = &handler_data.bot_data.accounts.personal_accounts[&handler_data.bot_data.users.users[candidate].account];
			action_row = action_row.with_components(
				Button::new()
					.with_style(ButtonStyle::Secondary)
					.with_label(&account.name)
					.with_custom_id(format!("vote{candidate}")),
			);
		}

		message = message.with_components(action_row);
	}

	respond_with_message(handler_data, message).await;
}

pub async fn view_results<'a>(handler_data: &mut HandlerData<'a>) {
	let previous_time = handler_data.bot_data.previous_time;
	let date = previous_time - Duration::hours((previous_time.weekday().number_from_sunday() % 7) as i64 * 24);
	let formatted_date = date.timestamp();
	let data = &handler_data.bot_data.previous_results;
	let description = format!("Results for election on <t:{formatted_date}:D> in csv format:\n```\n{data}```");
	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Election results").with_description(description),
	)
	.await;
}
pub async fn count_results<'a>(handler_data: &mut HandlerData<'a>) {
	let rolls = GuildMember::get_get_guild_member(handler_data.client, DiscordClient::GUILD_ID, &handler_data.user.id).await;
	let is_valid = rolls.as_ref().map_or(false, |user| user.roles.contains(&STAR_ROLE.to_string()));

	if !is_valid {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Count results").with_description("Incorrect rolly poly"),
		)
		.await;
		return;
	}

	let mut votes = handler_data.bot_data.election.iter().collect::<Vec<_>>();
	if votes.is_empty() {
		respond_with_embed(handler_data, Embed::standard().with_title("Count results").with_description("Empty")).await;
		handler_data.bot_data.save();
		return;
	}
	votes.sort_unstable_by_key(|v| -(v.1.len() as i32));
	for (user_id, votes) in votes {
		let cheese_user = handler_data.bot_data.users.get_mut(user_id).unwrap();
		let name = &handler_data.bot_data.accounts.personal_accounts[&cheese_user.account].name;
		handler_data.bot_data.previous_results += name;
		handler_data.bot_data.previous_results += ", ";
		handler_data.bot_data.previous_results += &votes.len().to_string();
		handler_data.bot_data.previous_results += "\n";
	}
	for (_, votes) in handler_data.bot_data.election.iter_mut() {
		*votes = Vec::new();
	}

	respond_with_embed(handler_data, Embed::standard().with_title("Count results").with_description("success!")).await;
}

pub async fn cast_vote<'a>(handler_data: &mut HandlerData<'a>, new_candidate: &str) {
	let embed = if handler_data.bot_data.election.contains_key(new_candidate) {
		let voter = &handler_data.user.id;
		for (candidate, votes) in &mut handler_data.bot_data.election {
			votes.retain(|e| e != voter);
			if candidate == new_candidate {
				votes.push(voter.clone());
			}
		}
		Embed::standard().with_title("Vote cast")
	} else {
		Embed::standard()
			.with_title("Vote failed")
			.with_description("Your candidate is no longer running")
	};
	respond_with_message(handler_data, ChannelMessage::new().with_embeds(embed).with_flags(1_u32 << 6)).await;
}
