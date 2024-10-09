use crate::bot_data::*;
use crate::utilities::*;
use chrono::Datelike;
use discord::*;
//pub const MP_ROLL: &str = "985804444237172797";
pub const STAR_ROLE: &str = "1171114445288779916";
pub const PRESIDENT_ROLL: &str = "907660552938061834";
/// Handles the `/about` command
pub async fn about<'a>(handler_data: &mut HandlerData<'a>) {
	respond_with_embed(
		handler_data,
		Embed::standard()
			.with_title("About")
			.with_description("This bot is developed by Go Consulting Ltd. to handle the finances of New New Cheeseland."),
	)
	.await;
}

/// Handles the `/balances` command
pub async fn balances<'a>(handler_data: &mut HandlerData<'a>) {
	fn format_account(Account { name, balance, .. }: &Account) -> String {
		format!("{:-20} {}\n", format!("{}:", name), format_cheesecoin(*balance))
	}

	let mut description = format!(
		"**Currency information**\n```\n{:-20} {}\n",
		"Total Currency:",
		format_cheesecoin(handler_data.bot_data.total_currency()),
	);

	for &(amount, tax_rate) in &handler_data.bot_data.wealth_tax {
		let limit = if amount == u32::MAX {
			" (no limit)".to_string()
		} else {
			format!(" <{}", format_cheesecoin(amount))
		};
		let _ = write!(&mut description, "{:-20} {:.2}%\n", format!("Balance Tax{}:", limit), tax_rate);
	}

	let _ = write!(&mut description, "{:-20} {:.2}%\n", "VAT", handler_data.bot_data.vat);
	//let _ = write!(&mut description, "{:-20} {}\n", "Tax", "removed via decree");
	let _ = write!(&mut description, "```\n**Your accounts**\n```");

	let cheese_user = handler_data.bot_data.cheese_user(&handler_data.user);

	// Add their personal account to the resulting string
	description += &format_account(&handler_data.bot_data.accounts.personal_accounts[&cheese_user.account]);

	// Add their organisations to the resulting string
	for account in &cheese_user.organisations {
		description += &format_account(&handler_data.bot_data.accounts.organisation_accounts[&account])
	}

	description += "```";

	respond_with_embed(handler_data, Embed::standard().with_title("Balances").with_description(description)).await;
}

/// Handles the `/pay` command
pub async fn pay<'a>(handler_data: &mut HandlerData<'a>) {
	let bot_data = &mut handler_data.bot_data;
	let recipient = match account_option(bot_data, &handler_data.options["recipient"], BotData::account_exists, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Payment").with_description("Invalid recipient."),
			)
			.await;
			return;
		}
	};
	let from = match account_option(bot_data, &handler_data.options["from"], BotData::account_owned, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(handler_data, Embed::standard().with_title("Payment").with_description("Invalid from.")).await;
			return;
		}
	};
	let amount = handler_data.options["cheesecoin"].as_float();

	let (payer_message, recipient_message) = transact(handler_data, recipient, from, amount);

	if let Some(message) = recipient_message {
		if let Some(recipient) = handler_data.bot_data.users.account_owner(recipient) {
			if let Err(e) = dm_embed(
				handler_data.client,
				Embed::standard().with_title("Payment").with_description(message),
				recipient,
			)
			.await
			{
				warn!("Payment dm failed {e:?}");
			}
		}
	}

	respond_with_embed(handler_data, Embed::standard().with_title("Payment").with_description(payer_message)).await;
}

/// Handles the `/sudo print cheesecoin` command
pub async fn print_money<'a>(handler_data: &mut HandlerData<'a>) {
	let bot_data = &mut handler_data.bot_data;

	let rolls = GuildMember::get_get_guild_member(handler_data.client, DiscordClient::GUILD_ID, &handler_data.user.id).await;
	let is_valid = rolls
		.as_ref()
		.map_or(false, |user| user.roles.contains(&"1293607752534593576".to_string()));

	if !is_valid {
		respond_with_embed(
			handler_data,
			Embed::standard()
				.with_title("Print money")
				.with_description("Super users only. Other users: buy a pony."),
		)
		.await;
		return;
	}

	let recipient = match account_option(bot_data, &handler_data.options["recipient"], BotData::account_exists, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Payment").with_description("Invalid recipient."),
			)
			.await;
			return;
		}
	};
	let amount = handler_data.options["cheesecoin"].as_float();

	let (payer_message, recipient_message) = enact_print_money(handler_data, recipient, amount);

	if let Some(message) = recipient_message {
		if let Some(recipient) = handler_data.bot_data.users.account_owner(recipient) {
			if let Err(e) = dm_embed(
				handler_data.client,
				Embed::standard().with_title("Money Printed for you").with_description(&message),
				recipient,
			)
			.await
			{
				warn!("Print money dm failed {e:?}");
			}
		}

		let embed = Embed::standard().with_title("Money Printed").with_description(message);

		if let Err(e) = ChannelMessage::new()
			.with_embeds(embed)
			.post_create(handler_data.client, "1171567720345649202")
			.await
		{
			error!("Failed to post print money update {e:?}");
		};
	}

	respond_with_embed(handler_data, Embed::standard().with_title("Print Money").with_description(payer_message)).await;
}

/// Handles the `/claim rollcall` command
pub async fn rollcall<'a>(handler_data: &mut HandlerData<'a>) {
	//let rolls = GuildMember::get_get_guild_member(handler_data.client, DiscordClient::GUILD_ID, &handler_data.user.id).await;
	// let is_mp = rolls.as_ref().map_or(false, |user| user.roles.contains(&MP_ROLL.to_string()));
	// let is_president = rolls.map_or(false, |user| user.roles.contains(&PRESIDENT_ROLL.to_string()));

	// if !is_mp {
	// 	let descripition = "You can only claim this benefit if you are an MP (if you are just ask to get the MP roll).";
	// 	respond_with_embed(
	// 		handler_data,
	// 		Embed::standard().with_title("Claim Rollcall").with_description(descripition),
	// 	)
	// 	.await;
	// 	return;
	// }

	let cheese_user = handler_data.bot_data.users.users.get_mut(&handler_data.user.id).unwrap();
	if cheese_user.last_pay.num_days_from_ce() == chrono::Utc::now().num_days_from_ce() {
		let descripition = format!(
			"You can claim this benefit only once per day. You have last claimed it {} hours ago.",
			(chrono::Utc::now() - cheese_user.last_pay).num_hours()
		);
		respond_with_disappear_embed(
			handler_data,
			Embed::standard().with_title("Claim Rollcall").with_description(descripition),
		)
		.await;
		return;
	}
	cheese_user.last_pay = chrono::Utc::now();

	let recipient = cheese_user.account;
	let amount = 2.; //if is_president { 4. } else { 2. };
	let (_, recipient_message) = transact(handler_data, recipient, TREASURY, amount);

	if let Some(message) = recipient_message {
		respond_with_disappear_embed(handler_data, Embed::standard().with_title("Claim Rollcall").with_description(message)).await;
	} else {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Claim Rollcall").with_description("Treasury Bankrupt!"),
		)
		.await;
	}
}
