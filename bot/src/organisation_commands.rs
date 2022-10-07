use crate::bot_data::*;
use crate::utilities::*;
use discord::*;

/// Handles the `/orgainsation create` command
pub async fn organisation_create<'a>(handler_data: &mut HandlerData<'a>) {
	let org_name = handler_data.options["name"].as_str();

	let name = org_name.clone();
	let account = Account {
		name,
		balance: 0,
		..Default::default()
	};

	handler_data
		.bot_data
		.accounts
		.organisation_accounts
		.insert(handler_data.bot_data.next_account, account);

	handler_data
		.bot_data
		.users
		.get_mut(&handler_data.user.id)
		.unwrap()
		.organisations
		.push(handler_data.bot_data.next_account);
	handler_data.bot_data.next_account += 1;

	let description = format!(
		"Successfully created {} which is owned by {}",
		org_name,
		handler_data.bot_data.personal_account_name(&handler_data.user)
	);

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Create Organisation").with_description(description),
	)
	.await;
}

pub async fn organisation_transfer<'a>(handler_data: &mut HandlerData<'a>) {
	let bot_data = &mut handler_data.bot_data;

	let organisation = match account_option(bot_data, &handler_data.options["name"], BotData::account_owned, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Transfer").with_description("Invalid organisation name"),
			)
			.await;
			return;
		}
	};

	let owner_account = match account_option(
		bot_data,
		&handler_data.options["owner"],
		BotData::personal_account_exists,
		&handler_data.user,
	)
	.await
	{
		Some(x) => x,
		None => {
			respond_with_embed(handler_data, Embed::standard().with_title("Payment").with_description("Invalid owner")).await;
			return;
		}
	};

	handler_data
		.bot_data
		.users
		.users
		.iter_mut()
		.find(|(_, user)| user.account == owner_account)
		.unwrap()
		.1
		.organisations
		.push(organisation);

	handler_data
		.bot_data
		.users
		.get_mut(&handler_data.user.id)
		.unwrap()
		.organisations
		.retain(|o| o != &organisation);

	let description = format!(
		"Transferred {} to {} successfully",
		handler_data.bot_data.accounts.organisation_accounts[&organisation].name,
		handler_data.bot_data.accounts.personal_accounts[&owner_account].name
	);

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Transferred organisation").with_description(description),
	)
	.await;
}

pub async fn organisation_rename<'a>(handler_data: &mut HandlerData<'a>) {
	let bot_data = &mut handler_data.bot_data;

	let organisation = match account_option(bot_data, &handler_data.options["name"], BotData::account_owned, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Rename").with_description("Invalid organisation name"),
			)
			.await;
			return;
		}
	};

	let org_name = handler_data.options["new"].as_str();

	let description = format!(
		"Renamed {} to {}",
		handler_data.bot_data.accounts.organisation_accounts[&organisation].name, org_name
	);

	handler_data.bot_data.accounts.organisation_accounts.get_mut(&organisation).unwrap().name = org_name;

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Renamed organisation").with_description(description),
	)
	.await;
}

pub async fn organisation_delete<'a>(handler_data: &mut HandlerData<'a>) {
	let bot_data = &mut handler_data.bot_data;

	let organisation = match account_option(bot_data, &handler_data.options["name"], BotData::account_owned, &handler_data.user).await {
		Some(x) => x,
		None => {
			respond_with_embed(
				handler_data,
				Embed::standard().with_title("Deletion").with_description("Invalid organisation name"),
			)
			.await;
			return;
		}
	};

	let description = format!("Deleted {}", handler_data.bot_data.accounts.organisation_accounts[&organisation].name);

	handler_data
		.bot_data
		.accounts
		.account_mut(handler_data.bot_data.cheese_user(&handler_data.user).account)
		.balance += handler_data.bot_data.accounts.organisation_accounts[&organisation].balance;

	handler_data
		.bot_data
		.users
		.get_mut(&handler_data.user.id)
		.unwrap()
		.organisations
		.retain(|o| o != &organisation);

	handler_data.bot_data.accounts.organisation_accounts.remove(&organisation);

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Deleted organisation").with_description(description),
	)
	.await;
}
