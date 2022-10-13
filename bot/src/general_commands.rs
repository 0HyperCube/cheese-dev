use crate::bot_data::*;
use crate::utilities::*;
use discord::*;

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
		"**Currency information**\n```\n{:-20} {}\n{:-20} {:.2}%\n```\n**Your accounts**\n```",
		"Total Currency:",
		format_cheesecoin(handler_data.bot_data.total_currency()),
		"Wealth Tax:",
		handler_data.bot_data.wealth_tax
	);

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
	let recipiant = match account_option(bot_data, &handler_data.options["recipiant"], BotData::account_exists, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Payment").with_description("Invalid recipiant."),
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

	let (payer_message, recipiant_message) = transact(handler_data, recipiant, from, amount);

	if let Some(message) = recipiant_message {
		dm_embed(
			handler_data.client,
			Embed::standard().with_title("Payment").with_description(message),
			handler_data.bot_data.users.account_owner(recipiant),
		)
		.await;
	}

	respond_with_embed(handler_data, Embed::standard().with_title("Payment").with_description(payer_message)).await;
}

/// Handles the `/claim rollcall` command
pub async fn rollcall<'a>(handler_data: &mut HandlerData<'a>) {
	const MP_ROLL: &str = "985804444237172797";

	let is_mp = GuildMember::get_get_guild_member(handler_data.client, DiscordClient::GUILD_ID, &handler_data.user.id)
		.await
		.map_or(false, |user| {
			info!("User {user:?}");
			user.roles.contains(&MP_ROLL.to_string())
		});

	if !is_mp {
		let descripition = "You can only claim this benefit if you are an MP (if you are just ask to get the MP roll).";
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Claim Rollcall").with_description(descripition),
		)
		.await;
		return;
	}

	let cheese_user = handler_data.bot_data.users.users.get_mut(&handler_data.user.id).unwrap();
	if chrono::Utc::now() - cheese_user.last_pay < chrono::Duration::hours(15) {
		let descripition = format!(
			"You can claim this benefit only once per day. You have last claimed it {} hours ago.",
			(chrono::Utc::now() - cheese_user.last_pay).num_hours()
		);
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Claim Rollcall").with_description(descripition),
		)
		.await;
		return;
	}
	cheese_user.last_pay = chrono::Utc::now();

	let recipiant = cheese_user.account;
	let (_, recipiant_message) = transact(handler_data, recipiant, TREASURY, 2.);

	if let Some(message) = recipiant_message {
		respond_with_embed(handler_data, Embed::standard().with_title("Claim Rollcall").with_description(message)).await;
	} else {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Claim Rollcall").with_description("Treasury Bankrupt!"),
		)
		.await;
	}
}