use crate::bot_data::*;
use crate::utilities::*;
use discord::*;

pub async fn create_bill(handler_data: &mut HandlerData<'_>) {
	let bill_name = handler_data.options["name"].as_str();

	let cheesecoin = handler_data.options["cheesecoin"].as_float() * 100.;
	if !cheesecoin.is_finite() || cheesecoin < 0. || cheesecoin >= u32::MAX as f64 {
		respond_with_embed(
			handler_data,
			Embed::standard()
				.with_title("Create Bill")
				.with_description("Amount must not be negative."),
		)
		.await;
		return;
	}
	let cheesecoin = cheesecoin as u32;

	let bot_data = &mut handler_data.bot_data;
	let to = match account_option(*bot_data, &handler_data.options["to"], BotData::account_owned, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(handler_data, Embed::standard().with_title("Create Bill").with_description("Invalid to.")).await;
			return;
		}
	};

	let days = handler_data.options["days"].as_float();
	if !days.is_finite() || days < 0. || days >= i32::MAX as f64 {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Create Bill").with_description("Days must not be negative."),
		)
		.await;
		return;
	}
	let days = days as i32;

	use chrono::Datelike;
	let bill = Bill {
		name: bill_name.clone(),
		last_pay: chrono::Utc::now().num_days_from_ce(),
		interval: days,
		amount: cheesecoin,
		owner: to,
		subscribers: Vec::new(),
	};

	let id = handler_data.bot_data.next_account;
	handler_data.bot_data.bills.insert(id, bill);

	handler_data.bot_data.accounts.account_mut(to).owned_bills.push(id);
	handler_data.bot_data.next_account += 1;

	let description = format!(
		"Successfully created {} which is owned by {}",
		bill_name,
		handler_data.bot_data.personal_account_name(&handler_data.user)
	);

	respond_with_embed(handler_data, Embed::standard().with_title("Create Bill").with_description(description)).await;
}

pub async fn bill_delete(handler_data: &mut HandlerData<'_>) {
	let bot_data = &mut handler_data.bot_data;

	let bill_id = match account_option(bot_data, &handler_data.options["name"], BotData::bill_owned, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Deletion").with_description("Invalid bill name"),
			)
			.await;
			return;
		}
	};

	let description = format!("Deleted {}", bot_data.bills.get(&bill_id).map_or("none", |bill| &bill.name));

	if let Some((bill_id, bill)) = bot_data.bills.remove_entry(&bill_id) {
		handler_data
			.bot_data
			.accounts
			.account_mut(bill.owner)
			.owned_bills
			.retain(|&v| v != bill_id);

		for subscriber in bill.subscribers {
			handler_data
				.bot_data
				.accounts
				.account_mut(subscriber)
				.subscribed_bills
				.retain(|&v| v != bill_id);
		}
	}

	respond_with_embed(handler_data, Embed::standard().with_title("Deleted bill").with_description(description)).await;
}

pub async fn bill_subscribe(handler_data: &mut HandlerData<'_>) {
	let bot_data = &mut handler_data.bot_data;

	let bill_id = match account_option(bot_data, &handler_data.options["name"], BotData::bill_exists, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Subscribe").with_description("Invalid bill name"),
			)
			.await;
			return;
		}
	};

	let from = match account_option(bot_data, &handler_data.options["from"], BotData::account_owned, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Subscribe").with_description("Invalid from account"),
			)
			.await;
			return;
		}
	};

	let bill = bot_data.bills.get_mut(&bill_id).unwrap();
	bill.subscribers.push(from);
	let owner = bot_data.accounts.account(bill.owner).name.clone();
	let bill_owner = format_bill(bill, owner);
	let from_account = bot_data.accounts.account_mut(from);
	from_account.subscribed_bills.push(bill_id);
	let description = format!("Subscribed to {} from account {}", bill_owner, from_account.name);

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Subscribed to bill").with_description(description),
	)
	.await;
}

pub async fn bill_unsubscribe(handler_data: &mut HandlerData<'_>) {
	let bot_data = &mut handler_data.bot_data;

	let bill_id = match account_option(bot_data, &handler_data.options["name"], BotData::bill_subscribed, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Unsubscribe").with_description("Invalid bill name"),
			)
			.await;
			return;
		}
	};
	let it = {
		let user = bot_data.cheese_user(&handler_data.user);
		user.organisations.iter().copied().chain([user.account]).collect::<Vec<_>>()
	};

	for account in it {
		let account = bot_data.accounts.account_mut(account);
		account.subscribed_bills.retain(|&x| x != bill_id)
	}

	let bill = bot_data.bills.get_mut(&bill_id).unwrap();
	let owner = account_immut(&bot_data.accounts.personal_accounts, &bot_data.accounts.organisation_accounts, bill.owner)
		.name
		.clone();
	let bill_owner = format_bill(bill, owner);
	let description = format!("Unsubscribed to {} ", bill_owner);

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Subscribed to bill").with_description(description),
	)
	.await;
}
