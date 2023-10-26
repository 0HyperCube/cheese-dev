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
	if !days.is_finite() || days < 1. || days >= i32::MAX as f64 {
		respond_with_embed(
			handler_data,
			Embed::standard()
				.with_title("Create Bill")
				.with_description("Days must not be less than one."),
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

	let Some(to_account) = handler_data.bot_data.accounts.account_mut(to) else {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Create Bill").with_description("Invalid account"),
		)
		.await;
		return;
	};
	to_account.owned_bills.push(id);
	handler_data.bot_data.next_account += 1;

	let description = format!(
		"successfully created {} which is owned by {}",
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
		if let Some(owner) = handler_data.bot_data.accounts.account_mut(bill.owner) {
			owner.owned_bills.retain(|&v| v != bill_id);
		}

		for subscriber in bill.subscribers {
			if let Some(subscriber) = handler_data.bot_data.accounts.account_mut(subscriber) {
				subscriber.subscribed_bills.retain(|&v| v != bill_id);
			}
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

	let Some(bill) = bot_data.bills.get_mut(&bill_id) else {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Subscribe").with_description("Invalid bill name"),
		)
		.await;
		return;
	};
	bill.subscribers.push(from);
	let owner = bot_data.accounts.account(bill.owner).map(|owner| owner.name.clone());
	let bill_owner = format_bill(bill, owner.clone().unwrap_or_else(|| "No owner".to_string()));
	let Some(from_account) = bot_data.accounts.account_mut(from) else {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Subscribe").with_description("Invalid account"),
		)
		.await;
		return;
	};
	from_account.subscribed_bills.push(bill_id);
	let description = format!("Subscribed to {} from account {}", bill_owner, from_account.name);

	if let Some(owner) = owner {
		dm_embed(
			handler_data.client,
			Embed::standard()
				.with_title("New subscriber")
				.with_description(format!("{} subscribed to your bill {}", from_account.name, bill_owner)),
			owner,
		)
		.await;
	}

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

	let user = bot_data.cheese_user(&handler_data.user);
	let user_owned_accounts = user.organisations.iter().copied().chain([user.account]).collect::<Vec<_>>();
	let Some(account) = bot_data.accounts.account(user.account) else {
		respond_with_embed(handler_data, Embed::standard().with_title("Unsubscribe").with_description("No account")).await;
		return;
	};
	let username = account.name.clone();

	for account in &user_owned_accounts {
		let Some(account) = bot_data.accounts.account_mut(*account) else {
			continue;
		};
		account.subscribed_bills.retain(|&x| x != bill_id)
	}

	let Some(bill) = bot_data.bills.get_mut(&bill_id) else {
		respond_with_embed(
			handler_data,
			Embed::standard().with_title("Unsubscribe").with_description("Invalid bill name"),
		)
		.await;
		return;
	};
	bill.subscribers.retain(|account| !user_owned_accounts.contains(account));

	let owner = bot_data.accounts.account(bill.owner).map(|owner| owner.name.clone());
	let bill_owner = format_bill(bill, owner.clone().unwrap_or_else(|| "No owner".to_string()));
	let description = format!("Unsubscribed to {} ", bill_owner);

	if let Some(owner) = owner {
		dm_embed(
			handler_data.client,
			Embed::standard()
				.with_title("Lost subscriber")
				.with_description(format!("{} unsubscribed to your bill {}", username, bill_owner)),
			owner,
		)
		.await;
	}

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Subscribed to bill").with_description(description),
	)
	.await;
}

pub async fn bill_view(handler_data: &mut HandlerData<'_>) {
	let bot_data = &mut handler_data.bot_data;

	let mut result = format!("**Bills your accounts are subscribed to**");
	let user_owned_accounts = {
		let user = bot_data.cheese_user(&handler_data.user);
		user.organisations.iter().copied().chain([user.account]).collect::<Vec<_>>()
	};

	let mut subscribed_bills = false;
	for account in &user_owned_accounts {
		let Some(account) = bot_data.accounts.account(*account) else {
			continue;
		};
		if account.subscribed_bills.is_empty() {
			continue;
		}
		let _ = write!(result, "\n{}:\n```", account.name);

		for bill_id in &account.subscribed_bills {
			if let Some(bill) = bot_data.bills.get(&bill_id) {
				let owner = bot_data
					.accounts
					.account(bill.owner)
					.map_or_else(|| "No owner".to_string(), |owner| owner.name.clone());
				let _ = write!(result, "\n{}", format_bill(bill, owner));
				subscribed_bills = true;
			}
		}
		let _ = write!(result, "\n```");
	}
	if !subscribed_bills {
		result += "\nNone";
	}

	let _ = write!(result, "\n\n**Owned Bills**");
	let mut owned_bills = false;
	for account in &user_owned_accounts {
		let Some(account) = bot_data.accounts.account(*account) else {
			continue;
		};
		for bill_id in &account.owned_bills {
			if let Some(bill) = bot_data.bills.get(&bill_id) {
				let _ = write!(result, "\n{}", format_bill(bill, account.name.clone()));
				result += "\nSubscribers:";
				result += "\n```";

				if bill.subscribers.is_empty() {
					result += "\nNone";
				} else {
					for subscriber in &bill.subscribers {
						result += "\n";
						result += &bot_data.accounts.account(*subscriber).map_or("No owner", |account| &account.name);
					}
				}

				owned_bills = true;
				result += "\n```";
			}
		}
	}
	if !owned_bills {
		result += "\nNone";
	}

	respond_with_embed(handler_data, Embed::standard().with_title("Your bills").with_description(result)).await;
}
