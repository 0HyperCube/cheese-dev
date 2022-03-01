use std::collections::HashMap;

use discord::{async_channel::Sender, *};
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate log;

type AccountId = u64;

/// The information tied to a specific discord userid
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CheeseUser {
	account: AccountId,
	mp: bool,
	last_pay: String,
	organisations: Vec<AccountId>,
}

/// Data about an accout (organisation or personal)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Account {
	name: String,
	balance: u32,
}

/// All the data the bot saves
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BotData {
	pub users: HashMap<String, CheeseUser>,
	pub personal_accounts: HashMap<AccountId, Account>,
	pub organisation_accounts: HashMap<AccountId, Account>,
	pub next_account: AccountId,
}

impl BotData {
	pub fn cheese_user<'a>(&'a self, user: &User) -> &'a CheeseUser {
		&self.users[&user.id]
	}

	pub fn personal_account_name(&self, user: &User) -> String {
		self.personal_accounts[&self.cheese_user(user).account].name.clone()
	}

	/// List all personal account names (with added suffix) and ids
	pub fn personal_accounts(&self) -> impl Iterator<Item = (String, AccountId)> + '_ {
		self.personal_accounts
			.iter()
			.map(|(id, account)| (account.name.clone() + " (Personal)", *id))
	}
	/// List all people names (with added suffix) and ids
	pub fn people(&self) -> impl Iterator<Item = (String, AccountId)> + '_ {
		self.personal_accounts
			.iter()
			.map(|(id, account)| (account.name.clone() + " (Person)", *id))
	}
	/// List all non-self people names (with added suffix) and ids
	pub fn non_self_people(&self, user: &User) -> impl Iterator<Item = (String, AccountId)> + '_ {
		let user = self.cheese_user(user);
		self.personal_accounts
			.iter()
			.filter(|(id, _)| **id == user.account)
			.map(|(id, account)| (account.name.clone() + " (Person)", *id))
	}
	/// List all organisation account names (with added suffix) and ids
	pub fn orgainsiation_accounts(&self) -> impl Iterator<Item = (String, AccountId)> + '_ {
		self.organisation_accounts
			.iter()
			.map(|(id, account)| (account.name.clone() + " (Organisation)", *id))
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
			.map(|(id, account)| (account.name.clone() + " (Organisation)", *id))
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

/// Construct the data handler (for implementing commands) from the specified interaction
///
/// This creates a new account if necessary, as well as flattenning subcommands into a space seperated string and finding the focused field
fn construct_handler_data<'a>(
	mut interaction: Interaction,
	client: &'a mut DiscordClient,
	bot_data: &'a mut BotData,
) -> (String, Option<InteractionDataOption>, HandlerData<'a>) {
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
				mp: true,
				last_pay: String::new(),
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

	// Extracts the command name (including sub commands)
	let mut options = data.options.take().unwrap_or(Vec::new());
	let mut command = data.name;
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

	(
		command,
		focused,
		HandlerData {
			client,
			bot_data,
			interaction,
			user,
			options,
		},
	)
}

/// Utility function for responding to an interaction with an embed
async fn respond_with_embed<'a>(handler_data: HandlerData<'a>, embed: Embed) {
	InteractionCallback::new(InteractionResponse::ChannelMessageWithSource {
		data: ChannelMessage::new().with_embeds(embed),
	})
	.post_respond(handler_data.client, handler_data.interaction.id, handler_data.interaction.token)
	.await
	.unwrap();
}

/// Handles the `/about` command
async fn about<'a>(handler_data: HandlerData<'a>) {
	respond_with_embed(
		handler_data,
		Embed::standard()
			.with_title("About")
			.with_description("This bot is developed by Go Consulting Ltd. to handle the finances of New New Cheeseland."),
	)
	.await;
}

/// Handles the `/balances` command
async fn balances<'a>(handler_data: HandlerData<'a>) {
	respond_with_embed(handler_data, Embed::standard().with_title("Balances")).await;
}

/// Handles the `/pay` command
async fn pay<'a>(handler_data: HandlerData<'a>) {
	respond_with_embed(handler_data, Embed::standard().with_title("Pay")).await;
}
/// Handles the `/orgainsation create` command
async fn organisation_create<'a>(handler_data: HandlerData<'a>) {
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
		"Sucessfully created {} which is owned by {}",
		org_name,
		handler_data.bot_data.personal_account_name(&handler_data.user)
	);

	respond_with_embed(
		handler_data,
		Embed::standard().with_title("Create Organisation").with_description(description),
	)
	.await;
}

async fn handle_interaction(interaction: Interaction, client: &mut DiscordClient, bot_data: &mut BotData) {
	let command_type = interaction.interaction_type.clone();
	let (command, focused, handler_data) = construct_handler_data(interaction, client, bot_data);
	match command_type {
		InteractionType::ApplicationCommand => {
			match command.as_str() {
				"about" => about(handler_data).await,
				"balances" => balances(handler_data).await,
				"pay" => pay(handler_data).await,
				"organisation create" => organisation_create(handler_data).await,
				_ => warn!("Unhandled command {}", command),
			};
		}
		InteractionType::ApplicationCommandAutocomplete => {
			let InteractionDataOption { name, value, .. } = focused.unwrap();
			let str_value = value.as_ref().unwrap().as_str().to_lowercase();
			info!("Autocomplete focused {} command {} value {}", name, command, str_value);

			let choices = match (command.as_str(), name.as_str()) {
				("pay", "recipiant") => handler_data
					.bot_data
					.personal_accounts()
					.chain(handler_data.bot_data.orgainsiation_accounts())
					.collect::<Vec<_>>(),
				("pay", "from") => handler_data
					.bot_data
					.personal_account(&handler_data.user)
					.chain(handler_data.bot_data.owned_orgs(&handler_data.user))
					.collect(),
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
						.with_name(name)
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
			return;
		}
	}
}

/// Continually tries to reconnect
async fn run_loop() {
	let mut client = DiscordClient::new(include_str!("token.txt"));

	// Open file and deserialise the data.
	let path = "cheese_data.ron";
	let mut bot_data = std::fs::read_to_string(path).map_or(BotData::default(), |v| ron::from_str(&v).unwrap_or(BotData::default()));
	// bot_data.personal_accounts.insert(
	// 	55,
	// 	Account {
	// 		name: "NewsCorp LTD".into(),
	// 		balance: 440,
	// 	},
	// );
	// bot_data.organisation_accounts.insert(
	// 	44,
	// 	Account {
	// 		name: "Bob the reporter".into(),
	// 		balance: 440,
	// 	},
	// );

	loop {
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

					info!("Recieved hello {:?}, sending identify {:?}", d, identify);

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
			MainMessage::WealthTax => todo!(),
			MainMessage::SaveFile => {
				info!("Saving data");
				std::fs::write(
					path,
					ron::ser::to_string_pretty(bot_data, ron::ser::PrettyConfig::new().indentor(String::from("\t"))).unwrap(),
				)
				.unwrap();
			}
		}
	}
}

async fn create_commands(client: &mut DiscordClient, application_id: &String) {
	ApplicationCommandList::new()
		.with_commands(
			ApplicationCommand::new()
				.with_command_type(CommandType::Chat)
				.with_name("about")
				.with_description("Description of the bot."),
		)
		.with_commands(
			ApplicationCommand::new()
				.with_command_type(CommandType::Chat)
				.with_name("balances")
				.with_description("All of your balances."),
		)
		.with_commands(
			ApplicationCommand::new()
				.with_command_type(CommandType::Chat)
				.with_name("pay")
				.with_description("Give someone cheesecoins.")
				.with_options(
					ApplicationCommandOption::new()
					.with_option_type(CommandOptionType::String)
						.with_name("recipiant").with_description("Recipiant of the payment")
						.with_required(true).with_autocomplete(true),
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
						.with_name("from").with_description("The account the cheesecoins are from")
						.with_required(true).with_autocomplete(true),
				)
		)
		.with_commands(
			ApplicationCommand::new()
				.with_command_type(CommandType::Chat)
				.with_name("organisation")
				.with_description("Organisation commands")
				.with_options(ApplicationCommandOption::new()
					.with_name("create")
					.with_description("Create an organisation.")
				.with_options(ApplicationCommandOption::new().with_option_type(CommandOptionType::String).with_name("name").with_required(true).with_description("The name of the new organisation"))),
		)
		// .with_commands(
		// 	ApplicationCommand::new()
		// 		.with_command_type(CommandType::Chat)
		// 		.with_name("")
		// 		.with_description(""),
		// )
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
