#![feature(int_roundings)]

use chrono::Datelike;
use discord::{async_channel::Sender, *};

mod bot_data;
use bot_data::*;

mod create_commands;
use create_commands::*;

mod utilities;
pub use utilities::*;

mod bill_commands;
mod general_commands;
mod organisation_commands;
mod parliament_commands;

#[macro_use]
extern crate log;

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
				"about" => general_commands::about(&mut handler_data).await,
				"balances" => general_commands::balances(&mut handler_data).await,
				"pay" => general_commands::pay(&mut handler_data).await,
				"organisation create" => organisation_commands::organisation_create(&mut handler_data).await,
				"organisation transfer" => organisation_commands::organisation_transfer(&mut handler_data).await,
				"organisation rename" => organisation_commands::organisation_rename(&mut handler_data).await,
				"organisation delete" => organisation_commands::organisation_delete(&mut handler_data).await,
				"claim rollcall" => general_commands::rollcall(&mut handler_data).await,
				"parliament run" => parliament_commands::set_running(&mut handler_data, true).await,
				"parliament stop running" => parliament_commands::set_running(&mut handler_data, false).await,
				"parliament vote" => parliament_commands::vote(&mut handler_data).await,
				"parliament view results" => parliament_commands::view_results(&mut handler_data).await,
				"bill create" => bill_commands::create_bill(&mut handler_data).await,
				"bill delete" => bill_commands::bill_delete(&mut handler_data).await,
				"bill subscribe" => bill_commands::bill_subscribe(&mut handler_data).await,
				"bill unsubscribe" => bill_commands::bill_unsubscribe(&mut handler_data).await,
				"bill view" => bill_commands::bill_view(&mut handler_data).await,
				_ => warn!("Unhandled command {}", command),
			};
		}
		InteractionType::MessageComponent => {
			if command.starts_with("vote") {
				parliament_commands::cast_vote(&mut handler_data, &command[4..]).await
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
				("pay", "from") | ("bill subscribe", "from") | ("bill create", "to") => handler_data
					.bot_data
					.personal_account(&handler_data.user)
					.chain(handler_data.bot_data.owned_orgs(&handler_data.user))
					.collect(),
				("organisation transfer", "name") | ("organisation rename", "name") | ("organisation delete", "name") => {
					handler_data.bot_data.owned_orgs(&handler_data.user).collect()
				}
				("organisation transfer", "owner") => handler_data.bot_data.non_self_people(&handler_data.user).collect(),
				("bill delete", "name") => handler_data.bot_data.owned_bills(&handler_data.user).collect(),
				("bill subscribe", "name") => handler_data.bot_data.bills().collect(),
				("bill unsubscribe", "name") => handler_data.bot_data.subscribed_bills(&handler_data.user).collect(),
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

fn save_file(bot_data: &mut BotData, path: &str) {
	if bot_data.changed {
		bot_data.changed = false;
		let new = ron::ser::to_string_pretty(bot_data, ron::ser::PrettyConfig::new().indentor(String::from("\t"))).unwrap();
		std::fs::write(path, new).unwrap();
	}
}

fn check_election(bot_data: &mut BotData) {
	let days_since = ((chrono::Utc::now() - bot_data.previous_time).num_hours()) / 24;
	let days_from_sunday = chrono::Utc::now().weekday().num_days_from_sunday();

	if days_from_sunday > 2 || days_since < 4 {
		return;
	}
	bot_data.previous_time = chrono::Utc::now();
	bot_data.previous_results = String::new();
	bot_data.changed = true;

	let mut votes = bot_data.election.iter().collect::<Vec<_>>();
	votes.sort_unstable_by_key(|v| -(v.1.len() as i32));
	for (user_id, votes) in votes {
		let cheese_user = bot_data.users.get_mut(user_id).unwrap();
		let name = &bot_data.accounts.personal_accounts[&cheese_user.account].name;
		bot_data.previous_results += name;
		bot_data.previous_results += ", ";
		bot_data.previous_results += &votes.len().to_string();
		bot_data.previous_results += "\n";
	}
	for (_, votes) in bot_data.election.iter_mut() {
		*votes = Vec::new();
	}
}

async fn check_wealth_tax(bot_data: &mut BotData, client: &mut DiscordClient) {
	if (chrono::Utc::now() - bot_data.last_wealth_tax) > chrono::Duration::hours(24 * 7 - 4) {
		bot_data.last_wealth_tax = bot_data.last_wealth_tax + chrono::Duration::hours(24 * 7);
		bot_data.changed = true;
		info!("Applying wealth tax.");

		// Applies welth tax to a specific account returning the log information for the user
		fn apply_wealth_tax_account(bot_data: &mut BotData, account: AccountId, name: Option<&str>, multiplier: f64) -> (String, u32) {
			let account = bot_data.accounts.account_mut(account);

			let tax = ((account.balance as f64 * multiplier).ceil()) as u32;
			account.balance -= tax;

			let result = format!(
				"\n{:20} -{:9} {}",
				name.unwrap_or(&account.name),
				format_cheesecoin(tax),
				format_cheesecoin(account.balance)
			);
			bot_data.treasury_account_mut().balance += tax;
			(result, tax)
		}

		let users = (&bot_data).users.users.keys().into_iter().map(|x| x.clone()).collect::<Vec<_>>();
		let mut total_tax = 0;

		for user_id in users {
			let origional_wealth = {
				bot_data.accounts.account(bot_data.users.users[&user_id].account).balance
					+ bot_data.users.users[&user_id]
						.organisations
						.clone()
						.iter()
						.map(|x| bot_data.accounts.account(*x).balance)
						.sum::<u32>()
			};

			let tax_rate = {
				let mut total_wealth = origional_wealth;
				let mut total_tax_collected = 0.;
				let paid = bot_data
					.wealth_tax
					.iter()
					.map(|&(amount, percent)| {
						// Amount of tax taken by that band
						let taxed = amount.min(total_wealth);
						total_wealth -= taxed;
						let tax_collected = taxed as f64 * percent / 100.;
						total_tax_collected += tax_collected;
						(taxed, percent)
					})
					.collect::<Vec<_>>();
				total_tax_collected / origional_wealth as f64
			};

			let mut result = format!("{:20} {:10} {}", "Account Name", "Tax", "New value");

			let (next, tax) = &apply_wealth_tax_account(bot_data, bot_data.users.users[&user_id].account.clone(), Some("Personal"), tax_rate);
			result += next;
			total_tax += tax;

			for org in bot_data.users.users[&user_id].organisations.clone() {
				if org == 0 {
					continue;
				}
				let (next, tax) = &apply_wealth_tax_account(bot_data, org, None, tax_rate);
				result += next;
				total_tax += tax;
			}

			if total_tax > 0 {
				let description = format!(
					"Wealth tax has been applied at `{}`.\n\n**Payments**\n```\n{}```",
					paid.into_iter()
						.filter(|(cc, _)| *cc != 0)
						.map(|(a, b)| format!("{}: {:.2}%", format_cheesecoin(a), b))
						.collect::<Vec<_>>()
						.join(","),
					result
				);

				dm_embed(
					client,
					Embed::standard().with_title("Wealth Tax").with_description(description),
					user_id.clone(),
				)
				.await;
			}
		}

		bot_data.wealth_taxes.push(total_tax);
		for (user_id, user) in &mut bot_data.users.users {
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

async fn check_bills(bot_data: &mut BotData, client: &mut DiscordClient) {
	for (_bill_id, bill) in &bot_data.bills {
		let mut bill_owner_result = String::new();
		let mut bill_owner_total = 0;
		let bill_owner_name = bot_data.accounts.account(bill.owner).name.clone();
		for &payer in &bill.subscribers {
			for _payment in 0..((bot_data.last_day - bill.last_pay).div_floor(bill.interval)) {
				let from = bot_data.accounts.account_mut(payer);

				if from.balance >= bill.amount {
					from.balance -= bill.amount;
					bill_owner_total += bill.amount;
					bill_owner_result += &format!("{:20} {}", from.name, format_cheesecoin(bill.amount));

					let sender_message = format!(
						"Successfully transfered {} from {} to {} in order to fund the bill {}.",
						format_cheesecoin(bill.amount),
						from.name,
						bill_owner_name,
						bill.name
					);

					let embed = Embed::standard()
						.with_title(format!("Paid {} for {} bill", format_cheesecoin(bill.amount), bill.name))
						.with_description(sender_message);
					dm_embed(client, embed, bot_data.users.account_owner(payer)).await;
				} else {
					let _ = write!(bill_owner_result, "{:20} Could not afford the bill", from.name);
					let sender_message = format!(
						"You failed to transfer {} from {} to {} in order to fund the bill {} due to insufficiant balance.",
						format_cheesecoin(bill.amount),
						from.name,
						bill_owner_name,
						bill.name
					);

					let embed = Embed::standard()
						.with_title(format!(
							"Could not afford {} bill - {} is unpaid",
							format_cheesecoin(bill.amount),
							bill.name
						))
						.with_description(sender_message);
					dm_embed(client, embed, bot_data.users.account_owner(payer)).await;
				}
			}
		}

		if (bot_data.last_day - bill.last_pay) >= bill.interval {
			let embed = Embed::standard()
				.with_title(format!("Collected {} from {} bill", format_cheesecoin(bill_owner_total), bill.name))
				.with_description(format!(
					"The bill {} has collected '{}' for {}.\n\n**Payments**\n```\n{}```",
					bill.name,
					format_cheesecoin(bill_owner_total),
					bot_data
						.accounts
						.organisation_accounts
						.get(&bill.owner)
						.map_or("your personal account", |org| &org.name),
					if bill_owner_result.is_empty() {
						format!("None")
					} else {
						format!("{:20} {}{}", "Account Name", "Charge", bill_owner_result)
					},
				));
			dm_embed(client, embed, bot_data.users.account_owner(bill.owner)).await;
		}
	}

	for bill in bot_data.bills.values_mut() {
		bill.last_pay += (bot_data.last_day - bill.last_pay).div_floor(bill.interval) * bill.interval;
	}
}

async fn treasury_balance(bot_data: &mut BotData, client: &mut DiscordClient) {
	let balance = bot_data.treasury_account().balance;
	let mut description = format!(
		"**Financial information for {}**\n```\n{:-20} {}\n",
		chrono::Utc::now().format("%d/%m/%Y"),
		"Total Currency:",
		format_cheesecoin(bot_data.total_currency()),
	);
	for &(amount, tax_rate) in &bot_data.wealth_tax {
		let limit = if amount == u32::MAX {
			" (no limit)".to_string()
		} else {
			format!(" <{}", format_cheesecoin(amount))
		};
		let _ = write!(&mut description, "{:-20} {:.2}%\n", format!("Wealth Tax{}:", limit), tax_rate);
	}
	let _ = write!(&mut description, "{:-20} {}\n```", "Treasury Balance:", format_cheesecoin(balance));
	bot_data.treasury_balances.push(balance);
	bot_data.changed = true;
	let embed = Embed::standard().with_title("Daily Treasury Report").with_description(description);

	ChannelMessage::new()
		.with_embeds(embed)
		.post_create(client, "1018447658685321266")
		.await
		.unwrap();
}
async fn twaddle(bot_data: &mut BotData, client: &mut DiscordClient) {
	let days = 738435 - bot_data.last_day;
	let embed = Embed::standard()
						.with_title(format!("Hangman in Rust due in {days} days!"))
						.with_description(format!("Dear Twaddle,\n\nI write to you today to inform you of an approaching deadline that it would be wise not to miss. The program which you yourself have willingly resolved to craft, Hangman in Rust, is due in **{days} days**. Whilst this deadline has the possibility of being percieved as tyranical, relentless and inhumane, I can, in all confidence, assure you that it will be exceptionally benificial to you and all of those around you.\n\nYou have long wanted to learn a low level systems programming language, and have spoken of pursuing a path involving scholarship in Rust for an imoderate period of time. Completing something like this will increase your motivation and perseverance as well as your attention span, which is vital to getting hired at the cheesecake factory which I percieve is your life aim.\n\nBest wishes,\nCheese Bot."));
	dm_embed(client, embed, "762325231925854231".to_string()).await;

	for images in [
		"https://i1.wp.com/www.pcfruit.com/wp-content/uploads/2017/04/20370.jpg?fit=4256%2C2832&ssl=1",
		"https://sherwoodphoenix.co.uk/wp-content/uploads/2016/04/Yamaha-C6-Boudoir-Grand-Piano-Black-Polyester-At-Sherwood-Phoenix-Pianos-1.jpg",
	] {
		let message = ChannelMessage::new().with_content(images);
		dm_message(client, message, "762325231925854231".to_string()).await;
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
				check_wealth_tax(bot_data, client).await;

				let day = chrono::Utc::now().num_days_from_ce();
				if day != bot_data.last_day {
					bot_data.last_day = day;
					treasury_balance(bot_data, client).await;
					twaddle(bot_data, client).await;
					check_bills(bot_data, client).await;
				}
			}
			MainMessage::SaveFile => save_file(bot_data, path),
			MainMessage::CheckElection => check_election(bot_data),
		}
	}
}

fn main() {
	init_logger();

	tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(run_loop());
}
