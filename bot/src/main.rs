#![feature(int_roundings)]
use std::collections::HashMap;

use chrono::{Datelike, Duration};
use discord::{async_channel::Sender, *};
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate log;

type AccountId = u64;
const TREASURY: AccountId = 0;

/// The information tied to a specific discord userid
#[derive(Debug, Serialize, Deserialize)]
pub struct CheeseUser {
	account: AccountId,
	last_pay: chrono::DateTime<chrono::Utc>,
	organisations: Vec<AccountId>,
}

/// Data about an accout (organisation or personal)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Account {
	name: String,
	balance: u32,
}

/// All the data the bot saves
#[derive(Debug, Serialize, Deserialize)]
pub struct BotData {
	pub users: HashMap<String, CheeseUser>,
	pub personal_accounts: HashMap<AccountId, Account>,
	pub organisation_accounts: HashMap<AccountId, Account>,
	pub next_account: AccountId,
	pub wealth_tax: f64,
	pub last_wealth_tax: chrono::DateTime<chrono::Utc>,
	pub election: HashMap<String, Vec<String>>,
	pub previous_time: chrono::DateTime<chrono::Utc>,
	pub previous_results: String,
	#[serde(skip)]
	pub changed: bool,
}

impl Default for BotData {
	fn default() -> Self {
		let organisation_accounts = HashMap::from([(
			0,
			Account {
				name: "Treasury".into(),
				balance: 1000,
			},
		)]);
		Self {
			users: HashMap::new(),
			personal_accounts: HashMap::new(),
			organisation_accounts,
			next_account: 1,
			wealth_tax: 0.05,
			last_wealth_tax: chrono::Utc::now(),
			election: HashMap::new(),
			previous_time: chrono::Utc::now(),
			previous_results: "No previous results".into(),
			changed: true,
		}
	}
}

impl BotData {
	/// Get the cheese user information given a discord user
	pub fn cheese_user<'a>(&'a self, user: &User) -> &'a CheeseUser {
		&self.users[&user.id]
	}

	/// Get the personal account name from a discord user
	pub fn personal_account_name(&self, user: &User) -> String {
		self.personal_accounts[&self.cheese_user(user).account].name.clone()
	}

	/// Get the account from an account id (either personal or organisation)
	pub fn account(&mut self, account: AccountId) -> &mut Account {
		self.personal_accounts
			.get_mut(&account)
			.map_or_else(|| self.organisation_accounts.get_mut(&account), |x| Some(x))
			.unwrap()
	}

	/// Checks if the given account id exists at all
	pub fn account_exists(&self, account: AccountId, _user: &User) -> bool {
		self.personal_accounts.contains_key(&account) || self.organisation_accounts.contains_key(&account)
	}

	/// Checks if the given personal account id exists at all
	pub fn personal_account_exists(&self, account: AccountId, _user: &User) -> bool {
		self.personal_accounts.contains_key(&account)
	}

	/// Checks if the given account id is owned by the specified user (personal or owned organisation)
	pub fn account_owned(&self, account: AccountId, user: &User) -> bool {
		let cheese_user = self.cheese_user(user);
		account == cheese_user.account || cheese_user.organisations.contains(&account)
	}

	/// Finds the account owner from an account id
	pub fn account_owner(&self, account: AccountId) -> String {
		self.users
			.iter()
			.find(|(_, user)| user.account == account || user.organisations.contains(&account))
			.unwrap()
			.0
			.clone()
	}

	/// Computes the total currency in circulation (for currency information in balances)
	pub fn total_currency(&self) -> u32 {
		self.personal_accounts.iter().map(|(_, a)| a.balance).sum::<u32>() + self.organisation_accounts.iter().map(|(_, a)| a.balance).sum::<u32>()
	}

	pub fn option_suffix(&self, id: &AccountId, default: &'static str) -> &'static str {
		match id {
			0 => " (Cheeselandic Government)",
			737928480389333004 => " (Dictator)",
			12 => " (Go Consulting Enteprises)",
			_ => default,
		}
	}

	/// List all personal account names (with added suffix) and ids
	pub fn personal_accounts(&self) -> impl Iterator<Item = (String, AccountId)> + '_ {
		self.personal_accounts
			.iter()
			.map(|(id, account)| (account.name.clone() + self.option_suffix(id, " (Personal)"), *id))
	}
	/// List all people names (with added suffix) and ids
	pub fn people(&self) -> impl Iterator<Item = (String, AccountId)> + '_ {
		self.personal_accounts
			.iter()
			.map(|(id, account)| (account.name.clone() + self.option_suffix(id, " (Person)"), *id))
	}
	/// List all non-self people names (with added suffix) and ids
	pub fn non_self_people(&self, user: &User) -> impl Iterator<Item = (String, AccountId)> + '_ {
		let user = self.cheese_user(user);
		self.personal_accounts
			.iter()
			.filter(|(id, _)| **id != user.account)
			.map(|(id, account)| (account.name.clone() + self.option_suffix(id, " (Person)"), *id))
	}
	/// List all organisation account names (with added suffix) and ids
	pub fn organisation_accounts(&self) -> impl Iterator<Item = (String, AccountId)> + '_ {
		self.organisation_accounts
			.iter()
			.map(|(id, account)| (account.name.clone() + self.option_suffix(id, " (Organisation)"), *id))
	}
	/// List the user's personal account as "Personal"
	pub fn personal_account(&self, user: &User) -> impl Iterator<Item = (String, AccountId)> + '_ {
		[("Personal".to_string(), self.cheese_user(user).account)].into_iter()
	}
	/// List all organisation account names the user owns (with added suffix) and ids
	pub fn owned_orgs(&self, user: &User) -> impl Iterator<Item = (String, AccountId)> + '_ {
		self.cheese_user(user)
			.organisations
			.iter()
			.map(|org| (org, &self.organisation_accounts[org]))
			.map(|(id, account)| (account.name.clone() + self.option_suffix(id, " (Organisation)"), *id))
	}
}

/// Data sent to a command handler
///
/// Includes the client, mutable access to bot data and the specified options to the command (with removed subcommands)
pub struct HandlerData<'a> {
	client: &'a mut DiscordClient,
	bot_data: &'a mut BotData,
	interaction: Interaction,
	user: User,
	options: HashMap<String, OptionType>,
}

// Use simplelog with a file and the console.
fn init_logger() {
	use simplelog::*;
	use std::fs::File;

	CombinedLogger::init(vec![
		TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
		WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("CheeseBot.log").unwrap()),
	])
	.unwrap();

	info!("Initalised logger!");
}

struct ConstructedData<'a> {
	command: String,
	focused: Option<InteractionDataOption>,
	handler_data: HandlerData<'a>,
}

/// Construct the data handler (for implementing commands) from the specified interaction
///
/// This creates a new account if necessary, as well as flattenning subcommands into a space seperated string and finding the focused field
fn construct_handler_data<'a>(mut interaction: Interaction, client: &'a mut DiscordClient, bot_data: &'a mut BotData) -> ConstructedData<'a> {
	// Extract the user from the interaction (if in guild, then interaction["member"]["user"], if in dms then interaction["user"])
	let user = (interaction)
		.user
		.as_ref()
		.unwrap_or_else(|| &interaction.member.as_ref().unwrap().user)
		.clone();

	// If the user does not already have an account, create a new one.
	if !bot_data.users.contains_key(&user.id) {
		bot_data.users.insert(
			user.id.clone(),
			CheeseUser {
				account: bot_data.next_account,
				last_pay: chrono::MIN_DATETIME,
				organisations: Vec::new(),
			},
		);
		bot_data.personal_accounts.insert(
			bot_data.next_account,
			Account {
				name: user.username.clone(),
				balance: 0,
			},
		);
		bot_data.next_account += 1;
	}

	let mut data = interaction.data.take().unwrap();

	if let Some(custom_id) = data.custom_id.take() {
		return ConstructedData {
			command: custom_id,
			focused: None,
			handler_data: HandlerData {
				client,
				bot_data,
				interaction,
				user,
				options: HashMap::new(),
			},
		};
	}
	// Extracts the command name (including sub commands)
	let mut options = data.options.take().unwrap_or(Vec::new());
	let mut command = data.name.unwrap();
	while options.len() > 0
		&& (options[0].option_type == CommandOptionType::SubCommandGroup || options[0].option_type == CommandOptionType::SubCommand)
	{
		command += " ";
		command += &options[0].name;
		options = options[0].options.take().unwrap_or(Vec::new());
	}

	// Extracts the focused field
	let focused = options.iter().find(|o| o.focused.unwrap_or(false)).map(|v| v.clone());

	// Extracts the options used
	let options = options.into_iter().map(|o| (o.name, o.value.unwrap())).collect::<HashMap<_, _>>();

	info!("Command name {}, options {:?}", command, options.keys());

	ConstructedData {
		command,
		focused,
		handler_data: HandlerData {
			client,
			bot_data,
			interaction,
			user,
			options,
		},
	}
}

/// Utility function for responding to an interaction with a message
async fn respond_with_message<'a>(handler_data: &mut HandlerData<'a>, message: ChannelMessage) {
	InteractionCallback::new(InteractionResponse::ChannelMessageWithSource { data: message })
		.post_respond(
			handler_data.client,
			handler_data.interaction.id.clone(),
			handler_data.interaction.token.clone(),
		)
		.await
		.unwrap();
}

/// Utility function for responding to an interaction with an embed
async fn respond_with_embed<'a>(handler_data: &mut HandlerData<'a>, embed: Embed) {
	respond_with_message(handler_data, ChannelMessage::new().with_embeds(embed)).await;
}

/// Utility function for dming a discord user an embed
async fn dm_embed<'a>(client: &mut DiscordClient, embed: Embed, recipient_id: String) {
	// We first create the channel (does nothing if it already exists)
	let channel = CreateDM { recipient_id }.post_create(client).await.unwrap();

	// Then we can send the message in the channel
	ChannelMessage::new().with_embeds(embed).post_create(client, channel.id).await.unwrap();
}

/// Utility function to extract an account from a slash command option
async fn account_option<'a, V>(bot_data: &mut BotData, option: &OptionType, validation: V, user: &User) -> Option<u64>
where
	V: Fn(&BotData, AccountId, &User) -> bool,
{
	let parse_int = str::parse::<AccountId>(&option.as_str());
	match parse_int.map(|id| (id, validation(bot_data, id, user))) {
		Ok((id, true)) => Some(id),
		_ => None,
	}
}

/// Handles the `/about` command
async fn about<'a>(handler_data: &mut HandlerData<'a>) {
	respond_with_embed(
		handler_data,
		Embed::standard()
			.with_title("About")
			.with_description("This bot is developed by Go Consulting Ltd. to handle the finances of New New Cheeseland."),
	)
	.await;
}

/// Handles the `/balances` command
async fn balances<'a>(handler_data: &mut HandlerData<'a>) {
	fn format_account(Account { name, balance }: &Account) -> String {
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
	description += &format_account(&handler_data.bot_data.personal_accounts[&cheese_user.account]);

	// Add their organisations to the resulting string
	for account in &cheese_user.organisations {
		description += &format_account(&handler_data.bot_data.organisation_accounts[&account])
	}

	description += "```";

	respond_with_embed(handler_data, Embed::standard().with_title("Balances").with_description(description)).await;
}

/// Utility function for formating cheesecoin as `4.23cc`
pub fn format_cheesecoin(cc: u32) -> String {
	format!("{:.2}cc", cc as f64 / 100.)
}

/// Handles transactions (from pay or mprollcall) - returns (payer message, reciever message)
fn transact<'a>(handler_data: &mut HandlerData<'a>, recipiant: u64, from: u64, amount: f64) -> (String, Option<String>) {
	// Special error for negitive
	if amount < 0. {
		return ("Cannot pay a negative amount.".into(), None);
	}
	// Amount cast into real units
	let amount = (amount * 100.) as u32;

	let from = handler_data.bot_data.account(from);

	// Check the account can back the transaction
	if from.balance < amount {
		return (format!("{} has only {}.", from.name, format_cheesecoin(from.balance)), None);
	}
	from.balance -= amount;
	let payer_name = from.name.clone();

	let recipiant = handler_data.bot_data.account(recipiant);
	recipiant.balance += amount;

	let reciever_message = format!(
		"Your account - {} - has recieved {} from {}.",
		recipiant.name,
		format_cheesecoin(amount),
		payer_name
	);

	let sender_message = format!(
		"Sucsessfully transfered {} from {} to {}.",
		format_cheesecoin(amount),
		payer_name,
		recipiant.name
	);

	(sender_message, Some(reciever_message))
}

/// Handles the `/pay` command
async fn pay<'a>(handler_data: &mut HandlerData<'a>) {
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
			handler_data.bot_data.account_owner(recipiant),
		)
		.await;
	}

	respond_with_embed(handler_data, Embed::standard().with_title("Payment").with_description(payer_message)).await;
}

/// Handles the `/claim rollcall` command
async fn rollcall<'a>(handler_data: &mut HandlerData<'a>) {
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

	let cheese_user = handler_data.bot_data.users.get_mut(&handler_data.user.id).unwrap();
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

/// Handles the `/parliament run` and `/parliament stop running` commands
async fn set_running<'a>(handler_data: &mut HandlerData<'a>, new_running: bool) {
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
async fn vote<'a>(handler_data: &mut HandlerData<'a>) {
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
			let account = &handler_data.bot_data.personal_accounts[&handler_data.bot_data.users[candidate].account];
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

async fn view_results<'a>(handler_data: &mut HandlerData<'a>) {
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

async fn cast_vote<'a>(handler_data: &mut HandlerData<'a>, new_candidate: &str) {
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

/// Handles the `/orgainsation create` command
async fn organisation_create<'a>(handler_data: &mut HandlerData<'a>) {
	let org_name = handler_data.options["name"].as_str();

	let name = org_name.clone();
	let account = Account { name, balance: 0 };

	handler_data
		.bot_data
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

async fn organisation_transfer<'a>(handler_data: &mut HandlerData<'a>) {
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
		handler_data.bot_data.organisation_accounts[&organisation].name, handler_data.bot_data.personal_accounts[&owner_account].name
	);

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Transferred organisation").with_description(description),
	)
	.await;
}

async fn organisation_rename<'a>(handler_data: &mut HandlerData<'a>) {
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
		handler_data.bot_data.organisation_accounts[&organisation].name, org_name
	);

	handler_data.bot_data.organisation_accounts.get_mut(&organisation).unwrap().name = org_name;

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Renamed organisation").with_description(description),
	)
	.await;
}

async fn organisation_delete<'a>(handler_data: &mut HandlerData<'a>) {
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

	let description = format!("Deleted {}", handler_data.bot_data.organisation_accounts[&organisation].name);

	handler_data
		.bot_data
		.account(handler_data.bot_data.cheese_user(&handler_data.user).account)
		.balance += handler_data.bot_data.organisation_accounts[&organisation].balance;

	handler_data
		.bot_data
		.users
		.get_mut(&handler_data.user.id)
		.unwrap()
		.organisations
		.retain(|o| o != &organisation);

	handler_data.bot_data.organisation_accounts.remove(&organisation);

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Deleted organisation").with_description(description),
	)
	.await;
}

async fn handle_interaction(interaction: Interaction, client: &mut DiscordClient, bot_data: &mut BotData) {
	bot_data.changed = true;
	let command_type = interaction.interaction_type.clone();
	let ConstructedData {
		command,
		focused,
		mut handler_data,
	} = construct_handler_data(interaction, client, bot_data);
	match command_type {
		InteractionType::ApplicationCommand => {
			match command.as_str() {
				"about" => about(&mut handler_data).await,
				"balances" => balances(&mut handler_data).await,
				"pay" => pay(&mut handler_data).await,
				"organisation create" => organisation_create(&mut handler_data).await,
				"organisation transfer" => organisation_transfer(&mut handler_data).await,
				"organisation rename" => organisation_rename(&mut handler_data).await,
				"organisation delete" => organisation_delete(&mut handler_data).await,
				"claim rollcall" => rollcall(&mut handler_data).await,
				"parliament run" => set_running(&mut handler_data, true).await,
				"parliament stop running" => set_running(&mut handler_data, false).await,
				"parliament vote" => vote(&mut handler_data).await,
				"parliament view results" => view_results(&mut handler_data).await,
				_ => warn!("Unhandled command {}", command),
			};
		}
		InteractionType::MessageComponent => {
			if command.starts_with("vote") {
				cast_vote(&mut handler_data, &command[4..]).await
			}
		}
		InteractionType::ApplicationCommandAutocomplete => {
			let InteractionDataOption { name, value, .. } = focused.unwrap();
			let str_value = value.as_ref().unwrap().as_str().to_lowercase();
			info!("Autocomplete focused {} command {} value {}", name, command, str_value);

			let choices = match (command.as_str(), name.as_str()) {
				("pay", "recipiant") => handler_data
					.bot_data
					.personal_accounts()
					.chain(handler_data.bot_data.organisation_accounts())
					.collect::<Vec<_>>(),
				("pay", "from") => handler_data
					.bot_data
					.personal_account(&handler_data.user)
					.chain(handler_data.bot_data.owned_orgs(&handler_data.user))
					.collect(),
				("organisation transfer", "name") | ("organisation rename", "name") | ("organisation delete", "name") => {
					handler_data.bot_data.owned_orgs(&handler_data.user).collect()
				}
				("organisation transfer", "owner") => handler_data.bot_data.non_self_people(&handler_data.user).collect(),
				_ => {
					warn!(r#"Invalid autocomplete for "{}" on command "{}""#, command, name);
					return;
				}
			};

			let choices = choices.into_iter()
				.filter(|(name, _)| name.to_lowercase().contains(&str_value))
				.enumerate()
				.filter(|(index, _)| *index < 25) // Discord does not allow >25 options.
				.map(|(_, value)| value)
				.map(|(name, id)| {
					ApplicationCommandOptionChoice::new()
						.with_name(&name[(name.len() as i32 -100).max(0) as usize ..name.len()])
						.with_value(OptionType::String(id.to_string()))
				})
				.collect::<Vec<_>>();

			InteractionCallback::new(InteractionResponse::ApplicationCommandAutocompleteResult {
				data: AutocompleteResult { choices },
			})
			.post_respond(handler_data.client, handler_data.interaction.id, handler_data.interaction.token)
			.await
			.unwrap();
		}
		_ => warn!("Recieved interaction of type {:?} which was not handled", command_type),
	}
}

#[derive(Clone)]
enum MainMessage {
	Gateway(GatewayRecieve),
	GatewayClosed,
	Heartbeat,
	WealthTax,
	SaveFile,
	CheckElection,
}

async fn read_websocket(mut read: Read, send_ev: Sender<MainMessage>) {
	while let Some(Ok(Message::Text(text))) = read.next().await {
		debug!("Recieved text {}", text);
		match serde_json::from_str(&text) {
			Ok(deserialised) => {
				if send_ev.send(MainMessage::Gateway(deserialised)).await.is_err() {
					return;
				}
			}
			Err(e) => {
				error!("Error decoding gateway message {:?}", e);
			}
		}
	}
	warn!("Websocket closing!");
	send_ev.send(MainMessage::GatewayClosed).await.unwrap_or(())
}

/// Sends a message every `period` milliseconds
async fn dispatch_msg(send_ev: Sender<MainMessage>, interval: u64, msg: MainMessage) {
	let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(interval));
	loop {
		interval.tick().await;
		if send_ev.send(msg.clone()).await.is_err() {
			warn!("Channel full");
			return;
		}
	}
}

/// Continually tries to reconnect
async fn run_loop() {
	let mut client = DiscordClient::new(include_str!("token.txt"));

	// Open file and deserialise the data.
	let path = "cheese_data.ron";
	let mut bot_data = std::fs::read_to_string(path).map_or(BotData::default(), |v| match ron::from_str(&v) {
		Err(e) => {
			error!("Decoding ron {:?}", e);
			panic!("Error decoding ron")
		}
		Ok(x) => x,
	});

	loop {
		warn!("Running in run loop");
		run(&mut client, &mut bot_data, path).await;
	}
}

/// Runs the bot
async fn run(client: &mut DiscordClient, bot_data: &mut BotData, path: &str) {
	let gateway = GatewayMeta::get_gateway_meta(client).await.unwrap();
	info!("Recieved gateway metadata: {:?}", gateway);

	let (send_ev, mut recieve_ev) = async_channel::unbounded();

	let Connection { send_outgoing_message, read } = client.connect_gateway(gateway.url).await;

	let mut sequence_number = None;

	tokio::spawn(read_websocket(read, send_ev.clone()));

	tokio::spawn(dispatch_msg(send_ev.clone(), 60000, MainMessage::SaveFile));
	tokio::spawn(dispatch_msg(send_ev.clone(), 60000, MainMessage::WealthTax));
	tokio::spawn(dispatch_msg(send_ev.clone(), 60000, MainMessage::CheckElection));

	while let Some(main_message) = recieve_ev.next().await {
		match main_message {
			MainMessage::Gateway(deserialised) => match deserialised {
				GatewayRecieve::Dispatch { d, s } => {
					sequence_number = Some(s);

					debug!("Recieved dispatch {:?}", d);
					match d {
						Dispatch::Ready(r) => create_commands(client, &r.application.id).await,
						Dispatch::InteractionCreate(interaction) => handle_interaction(interaction, client, bot_data).await,
						_ => warn!("Unhandled dispatch"),
					}
				}
				GatewayRecieve::Heartbeat { .. } => {
					warn!("Discord wants a heartbeat, sending (should probably not happen)");
					send_ev.send(MainMessage::Heartbeat).await.unwrap();
				}
				GatewayRecieve::Reconnect => {
					warn!("Discord has told us to reconnect");
					return;
				}
				GatewayRecieve::InvalidSession { d } => error!("Invalid session, can reconnect {}", d),
				GatewayRecieve::Hello { d } => {
					let identify = GatewaySend::Identify {
						d: Identify::new()
							.with_intents(INTENTS_NONE)
							.with_token(&client.token)
							.with_properties(ConnectionProperties::new().with_device("Cheese")),
					};

					send_outgoing_message.send(serde_json::to_string(&identify).unwrap()).await.unwrap();
					tokio::spawn(dispatch_msg(send_ev.clone(), d.heartbeat_interval, MainMessage::Heartbeat));
				}
				GatewayRecieve::HeartbeatACK => {}
			},
			MainMessage::GatewayClosed => return,
			MainMessage::Heartbeat => {
				send_outgoing_message
					.send(serde_json::to_string(&GatewaySend::Heartbeat { d: sequence_number }).unwrap())
					.await
					.unwrap();
			}
			MainMessage::WealthTax => {
				if (chrono::Utc::now() - bot_data.last_wealth_tax) > chrono::Duration::hours(24 * 7 - 4) {
					bot_data.last_wealth_tax = bot_data.last_wealth_tax + chrono::Duration::hours(24 * 7);
					bot_data.changed = true;
					info!("Applying wealth tax.");

					// Applies welth tax to a specific account returning the log information for the user
					fn apply_wealth_tax_account(bot_data: &mut BotData, account: AccountId, name: Option<&str>) -> (String, u32) {
						let multiplier = bot_data.wealth_tax / 100.;
						let account = bot_data.account(account);
						let tax = ((account.balance as f64 * multiplier).ceil()) as u32;
						account.balance -= tax;

						let result = format!(
							"\n{:20} -{:9} {}",
							name.unwrap_or(&account.name),
							format_cheesecoin(tax),
							format_cheesecoin(account.balance)
						);
						bot_data.organisation_accounts.get_mut(&TREASURY).unwrap().balance += tax;
						(result, tax)
					}

					let users = (&bot_data).users.keys().into_iter().map(|x| x.clone()).collect::<Vec<_>>();
					let mut total_tax = 0;

					for user_id in users {
						let mut result = format!("{:20} {:10} {}", "Account Name", "Tax", "New value");

						let (next, tax) = &apply_wealth_tax_account(bot_data, bot_data.users[&user_id].account.clone(), Some("Personal"));
						result += next;
						total_tax += tax;

						for org in bot_data.users[&user_id].organisations.clone() {
							if org == 0 {
								continue;
							}
							let (next, tax) = &apply_wealth_tax_account(bot_data, org, None);
							result += next;
							total_tax += tax;
						}

						let description = format!(
							"Wealth tax has been applied at `{:.2}%`.\n\n**Payments**\n```\n{}```",
							bot_data.wealth_tax, result
						);

						dm_embed(
							client,
							Embed::standard().with_title("Wealth Tax").with_description(description),
							user_id.clone(),
						)
						.await;
					}

					for (user_id, user) in &mut bot_data.users {
						if user.organisations.contains(&0) {
							let description = format!("The treasury has collected {} of wealth tax.", format_cheesecoin(total_tax));

							dm_embed(
								client,
								Embed::standard().with_title("Total Wealth Tax").with_description(description),
								user_id.clone(),
							)
							.await;
							break;
						}
					}
				}
			}
			MainMessage::SaveFile => {
				if bot_data.changed {
					bot_data.changed = false;
					let new = ron::ser::to_string_pretty(bot_data, ron::ser::PrettyConfig::new().indentor(String::from("\t"))).unwrap();
					std::fs::write(path, new).unwrap();
				}
			}
			MainMessage::CheckElection => {
				let days_since = ((chrono::Utc::now() - bot_data.previous_time).num_hours()) / 24;
				let days_from_sunday = chrono::Utc::now().weekday().num_days_from_sunday();

				if days_from_sunday > 2 || days_since < 4 {
					continue;
				}
				bot_data.previous_time = chrono::Utc::now();
				bot_data.previous_results = String::new();

				let mut votes = bot_data.election.iter().collect::<Vec<_>>();
				votes.sort_unstable_by_key(|v| -(v.1.len() as i32));
				for (user_id, votes) in votes {
					let cheese_user = bot_data.users.get_mut(user_id).unwrap();
					let name = &bot_data.personal_accounts[&cheese_user.account].name;
					bot_data.previous_results += name;
					bot_data.previous_results += ", ";
					bot_data.previous_results += &votes.len().to_string();
					bot_data.previous_results += "\n";
				}
				for (_, votes) in bot_data.election.iter_mut() {
					*votes = Vec::new();
				}
			}
		}
	}
}

async fn create_commands(client: &mut DiscordClient, application_id: &String) {
	let about = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("about")
		.with_description("Description of the bot.");
	let balances = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("balances")
		.with_description("All of your balances.");
	let pay = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("pay")
		.with_description("Give someone cheesecoins.")
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::String)
				.with_name("recipiant")
				.with_description("Recipiant of the payment")
				.with_required(true)
				.with_autocomplete(true),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::Number)
				.with_name("cheesecoin")
				.with_description("Number of cheesecoin")
				.with_required(true),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::String)
				.with_name("from")
				.with_description("The account the cheesecoins are from")
				.with_required(true)
				.with_autocomplete(true),
		);
	let organisation = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("organisation")
		.with_description("Organisation commands")
		.with_options(
			ApplicationCommandOption::new()
				.with_name("create")
				.with_description("Create an organisation.")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the new organisation"),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("transfer")
				.with_description("Transfer an organisation")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the organisation")
						.with_autocomplete(true),
				)
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("owner")
						.with_required(true)
						.with_description("The new owner of the organisation")
						.with_autocomplete(true),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("rename")
				.with_description("Rename an organisation")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the organisation")
						.with_autocomplete(true),
				)
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("new")
						.with_required(true)
						.with_description("The new name of the organisation"),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("delete")
				.with_description("Delete an organisation")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the organisation")
						.with_autocomplete(true),
				),
		);
	let rollcall = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("claim")
		.with_description("Claim commands")
		.with_options(
			ApplicationCommandOption::new()
				.with_name("rollcall")
				.with_description("Claim your MP daily rollcall"),
		);
	let parliament = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("parliament")
		.with_description("Parliament commands")
		.with_options(ApplicationCommandOption::new().with_name("run").with_description("Run as candidate."))
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::SubCommandGroup)
				.with_name("stop")
				.with_description("Stop doing something.")
				.with_options(
					ApplicationCommandOption::new()
						.with_name("running")
						.with_description("Stop running as candidate"),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("vote")
				.with_description("Vote for a candidate (or change your vote)."),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::SubCommandGroup)
				.with_name("view")
				.with_description("View something.")
				.with_options(
					ApplicationCommandOption::new()
						.with_name("results")
						.with_description("View results of last election."),
				),
		);
	ApplicationCommandList::new()
		.with_commands(about)
		.with_commands(balances)
		.with_commands(pay)
		.with_commands(rollcall)
		.with_commands(organisation)
		.with_commands(parliament)
		.put_bulk_override_global(client, application_id)
		.await
		.unwrap();
}

fn main() {
	init_logger();

	tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(run_loop());
}
