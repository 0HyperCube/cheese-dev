#![feature(int_roundings)]
#![feature(panic_update_hook)]
#![feature(map_many_mut)]
use std::panic;

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
mod role_commands;

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

	panic::update_hook(move |prev, info: &panic::PanicInfo<'_>| {
		info!("{:?}", info.to_string());
		prev(info);
	});
}

async fn handle_interaction(interaction: Interaction, client: &mut DiscordClient, bot_data: &mut BotData) {
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
				"role assign" => role_commands::role_assign(&mut handler_data).await,
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
		_ => warn!("received interaction of type {:?} which was not handled", command_type),
	}
	bot_data.save();
}

#[derive(Clone)]
enum MainMessage {
	Gateway(GatewayRecieve),
	GatewayClosed,
	Heartbeat,
	WealthTax,
	CheckElection,
}

async fn read_websocket(mut read: Read, send_ev: Sender<MainMessage>) {
	while let Some(Ok(Message::Text(text))) = read.next().await {
		debug!("received text {}", text);
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

#[test]
fn decode_gateway() {
	let x:GatewayRecieve = serde_json::from_str(r##"{"t":"INTERACTION_CREATE","s":36,"op":0,"d":{"version":1,"user":{"username":"Käse","public_flags":128,"id":"630073509137350690","display_name":null,"discriminator":"3615","avatar_decoration":null,"avatar":"761226738b7c90394e301b7f387fdc9d"},"type":3,"token":"aW50ZXJhY3Rpb246MTA3ODA2ODQ5Mzc4Nzg1NjkzODplVkhFUnZweEZNYkxqWGI2ZmdRVHNJUUY2UUxrZHViM3RMbWl1SVExNWhLUlAxQTNvQ0dKRDBBMnhiUmFVc29RZFI5RnAwUG4xTTRzVEROYzdnam5USnRKUUc2cm1LMkZNVnJtZmY5S1NTQ0tKVGhHN1lhU1V1VTVSdEpmQ2xvMg","message":{"webhook_id":"910254320740610069","type":20,"tts":false,"timestamp":"2023-02-22T21:39:17.323000+00:00","pinned":false,"mentions":[],"mention_roles":[],"mention_everyone":false,"interaction":{"user":{"username":"Käse","public_flags":128,"id":"630073509137350690","display_name":null,"discriminator":"3615","avatar_decoration":null,"avatar":"761226738b7c90394e301b7f387fdc9d"},"type":2,"name":"parliament vote","id":"1078068487651606608"},"id":"1078068489006358578","flags":0,"embeds":[],"edited_timestamp":null,"content":"Vote for your candidate:","components":[{"type":1,"components":[{"type":2,"style":2,"label":"Alan","custom_id":"vote887020108696920116"},{"type":2,"style":2,"label":"аe","custom_id":"vote762325231925854231"},{"type":2,"style":2,"label":"Elliot F.","custom_id":"vote722468356711776269"}]}],"channel_id":"910597009466093628","author":{"username":"Cheese Bot","public_flags":0,"id":"910254320740610069","display_name":null,"discriminator":"4538","bot":true,"avatar_decoration":null,"avatar":null},"attachments":[],"application_id":"910254320740610069"},"locale":"en-GB","id":"1078068493787856938","data":{"custom_id":"vote887020108696920116","component_type":2},"channel_id":"910597009466093628","application_id":"910254320740610069"}}"##).unwrap();
	let x:GatewayRecieve = serde_json::from_str(r##"{"t":"INTERACTION_CREATE","s":32,"op":0,"d":{"version":1,"type":3,"token":"aW50ZXJhY3Rpb246MTA3ODA2MzY2OTg3Njg4NzY1Mjo2RDRLVUg2c1FvSTlSa2w0Q2VkR3NMelhiN2pXMUpDMzNWT2VMeTlDZzEyZWUyWWRNc2JWd1psdWJsOE5GZzBCTGZnYUM0OXEzZ1E1eHJxZFBEb3ZDY1o5RTlBa2RHNzJyUGZ0ejVaeHd5R1dpQ0lXYVNpWFVuUVBoaEo5eDY2NA","message":{"webhook_id":"910254320740610069","type":20,"tts":false,"timestamp":"2023-02-22T21:11:38.400000+00:00","pinned":false,"mentions":[],"mention_roles":[],"mention_everyone":false,"interaction":{"user":{"username":"Käse","public_flags":128,"id":"630073509137350690","display_name":null,"discriminator":"3615","avatar_decoration":null,"avatar":"761226738b7c90394e301b7f387fdc9d"},"type":2,"name":"parliament vote","id":"1078061529682956439"},"id":"1078061530978992240","flags":0,"embeds":[],"edited_timestamp":null,"content":"Vote for your candidate:","components":[{"type":1,"components":[{"type":2,"style":2,"label":"Alan","custom_id":"vote887020108696920116"},{"type":2,"style":2,"label":"аe","custom_id":"vote762325231925854231"},{"type":2,"style":2,"label":"Elliot F.","custom_id":"vote722468356711776269"}]}],"channel_id":"956233767767408741","author":{"username":"Cheese Bot","public_flags":0,"id":"910254320740610069","display_name":null,"discriminator":"4538","bot":true,"avatar_decoration":null,"avatar":null},"attachments":[],"application_id":"910254320740610069"},"member":{"user":{"username":"18XYang","public_flags":0,"id":"764206694841974795","display_name":null,"discriminator":"6684","avatar_decoration":null,"avatar":null},"roles":["1078035383511699496","985630968650010705","985804444237172797"],"premium_since":null,"permissions":"539018919105","pending":false,"nick":"Xiao-Kun","mute":false,"joined_at":"2023-02-22T19:00:46.675000+00:00","is_pending":false,"flags":0,"deaf":false,"communication_disabled_until":null,"avatar":null},"locale":"en-GB","id":"1078063669876887652","guild_locale":"en-US","guild_id":"907657508292792342","entitlement_sku_ids":[],"data":{"custom_id":"vote887020108696920116","component_type":2},"channel_id":"956233767767408741","application_id":"910254320740610069","app_permissions":"4398046511103"}}"##).unwrap();
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
	let mut client = DiscordClient::new(include_str!("token.txt").trim());

	// Open file and deserialise the data.
	let path = "cheese_data.ron";
	std::fs::copy(path, format!("cheese_data_back_{}.ron", chrono::Utc::now().num_days_from_ce())).unwrap();
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

fn check_election(bot_data: &mut BotData) {
	let days_since = ((chrono::Utc::now() - bot_data.previous_time).num_hours()) / 24;
	let days_from_sunday = chrono::Utc::now().weekday().num_days_from_sunday();

	if  chrono::Utc::now().num_days_from_ce() == bot_data.previous_time.num_days_from_ce()
		 || (((days_from_sunday > 2 && days_from_sunday != 6) || days_since < 4)
		&& chrono::Utc::now().num_days_from_ce() != 738838)
	{
		return;
	}
	bot_data.previous_time = chrono::Utc::now();
	bot_data.previous_results = String::new();

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
	bot_data.save();
}

async fn check_wealth_tax(bot_data: &mut BotData, client: &mut DiscordClient) {
	if (chrono::Utc::now() - bot_data.last_wealth_tax) <= chrono::Duration::hours(24 * 7 - 4) {
		return;
	}

	bot_data.last_wealth_tax = bot_data.last_wealth_tax + chrono::Duration::hours(24 * 7);
	info!("Applying wealth tax.");

	// Applies welth tax to a specific account returning the log information for the user
	fn apply_wealth_tax_account(bot_data: &mut BotData, account: AccountId, name: Option<&str>, multiplier: f64) -> Option<(String, u32)> {
		let account = bot_data.accounts.account_mut(account)?;

		let tax = ((account.balance as f64 * multiplier).ceil()) as u32;
		account.balance -= tax;

		let result = format!(
			"\n{:20} -{:9} {}",
			name.unwrap_or(&account.name),
			format_cheesecoin(tax),
			format_cheesecoin(account.balance)
		);
		bot_data.treasury_account_mut().balance += tax;
		Some((result, tax))
	}

	let users = (&bot_data).users.users.keys().into_iter().map(|x| x.clone()).collect::<Vec<_>>();
	let mut total_tax = 0;

	for user_id in users {
		let origional_wealth = {
			bot_data
				.accounts
				.account(bot_data.users.users[&user_id].account)
				.map_or(0, |account| account.balance)
				+ bot_data.users.users[&user_id]
					.organisations
					.clone()
					.iter()
					.map(|x| bot_data.accounts.account(*x).map_or(0, |account| account.balance))
					.sum::<u32>()
		};

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
		let tax_rate = total_tax_collected / origional_wealth as f64;

		let mut result = format!("{:20} {:10} {}", "Account Name", "Tax", "New value");

		let tax = &apply_wealth_tax_account(bot_data, bot_data.users.users[&user_id].account.clone(), Some("Personal"), tax_rate);
		result += tax.as_ref().map_or(&"", |tax| &tax.0);
		total_tax += tax.as_ref().map_or(0, |tax| tax.1);

		for org in bot_data.users.users[&user_id].organisations.clone() {
			if org == 0 {
				continue;
			}
			let tax = &apply_wealth_tax_account(bot_data, org, None, tax_rate);
			result += tax.as_ref().map_or(&"", |tax| &tax.0);
			total_tax += tax.as_ref().map_or(0, |tax| tax.1);
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

			if let Err(e) = dm_embed(
				client,
				Embed::standard().with_title("Wealth Tax").with_description(description),
				user_id.clone(),
			)
			.await
			{
				error!("Failed to dm {user_id} about their wealth tax: {e:?}.")
			}
		}
	}

	bot_data.wealth_taxes.push(total_tax);
	for (user_id, user) in &mut bot_data.users.users {
		if user.organisations.contains(&0) {
			let description = format!("The treasury has collected {} of wealth tax.", format_cheesecoin(total_tax));
			if let Err(e) = dm_embed(
				client,
				Embed::standard().with_title("Total Wealth Tax").with_description(description),
				user_id.clone(),
			)
			.await
			{
				error!("Failed to message treasury owner {user_id} about the collected wealth tax: {e:?}");
			};
			break;
		}
	}

	bot_data.save();
}

async fn check_bills(bot_data: &mut BotData, client: &mut DiscordClient) {
	for (_bill_id, bill) in &bot_data.bills {
		let mut bill_owner_result = String::new();
		let mut bill_owner_total = 0;
		let Some(bill_owner) = bot_data.accounts.account(bill.owner) else {
			continue;
		};
		let bill_owner_name = bill_owner.name.clone();
		for &payer in &bill.subscribers {
			for _payment in 0..((bot_data.last_day - bill.last_pay).div_floor(bill.interval)) {
				let Some(from) = bot_data.accounts.account_mut(payer) else {
					continue;
				};

				bill_owner_result += "\n";
				if from.balance >= bill.amount {
					from.balance -= bill.amount;
					bill_owner_total += bill.amount;
					bill_owner_result += &format!("{:20} {}", from.name, format_cheesecoin(bill.amount));

					let sender_message = format!(
						"successfully transfered {} from {} to {} in order to fund the bill {}.",
						format_cheesecoin(bill.amount),
						from.name,
						bill_owner_name,
						bill.name
					);

					if let Some(payer) = bot_data.users.account_owner(payer) {
						let embed = Embed::standard()
							.with_title(format!("Paid {} for {} bill", format_cheesecoin(bill.amount), bill.name))
							.with_description(sender_message);
						if let Err(e) = dm_embed(client, embed, payer.clone()).await {
							error!("Failed to dm bill payer {payer} about payment: {e:?}");
						}
					}
				} else {
					let _ = write!(bill_owner_result, "{:20} Could not afford the bill", from.name);
					let sender_message = format!(
						"You failed to transfer {} from {} to {} in order to fund the bill {} due to insufficiant balance.",
						format_cheesecoin(bill.amount),
						from.name,
						bill_owner_name,
						bill.name
					);

					if let Some(payer) = bot_data.users.account_owner(payer) {
						let embed = Embed::standard()
							.with_title(format!(
								"Could not afford {} bill - {} is unpaid",
								format_cheesecoin(bill.amount),
								bill.name
							))
							.with_description(sender_message);
						if let Err(e) = dm_embed(client, embed, payer.clone()).await {
							error!("Failed to send failiure to transfer message to {payer}: {e:?}");
						}
					}
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
			if let Some(recipiant) = bot_data.users.account_owner(bill.owner) {
				if let Err(e) = dm_embed(client, embed, recipiant.clone()).await {
					error!("Failed to send collected bill to {recipiant} error: {e:?}");
				}
			}
			if let Some(owner) = bot_data.accounts.account_mut(bill.owner) {
				owner.balance += bill_owner_total;
			}
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
	bot_data.save();
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
	if let Err(e) = dm_embed(client, embed, "762325231925854231".to_string()).await {
		error!("Twaddle :( {e:?}");
	}

	for images in [
		"https://i1.wp.com/www.pcfruit.com/wp-content/uploads/2017/04/20370.jpg?fit=4256%2C2832&ssl=1",
		"https://sherwoodphoenix.co.uk/wp-content/uploads/2016/04/Yamaha-C6-Boudoir-Grand-Piano-Black-Polyester-At-Sherwood-Phoenix-Pianos-1.jpg",
	] {
		let message = ChannelMessage::new().with_content(images);
		if let Err(e) = dm_message(client, message, "762325231925854231".to_string()).await {
			error!("Twaddle :( {e:?}");
		}
	}
}

/// Runs the bot
async fn run(client: &mut DiscordClient, bot_data: &mut BotData, path: &str) {
	bot_data.file_path = path.to_string();
	let gateway = GatewayMeta::get_gateway_meta(client).await.unwrap();
	info!("received gateway metadata: {:?}", gateway);

	let (send_ev, mut recieve_ev) = async_channel::unbounded();

	let Some(Connection { send_outgoing_message, read }) = client.connect_gateway(gateway.url).await else {
		return;
	};

	let mut sequence_number = None;

	tokio::spawn(read_websocket(read, send_ev.clone()));

	tokio::spawn(dispatch_msg(send_ev.clone(), 3 * 60 * 60 * 1000, MainMessage::WealthTax));
	tokio::spawn(dispatch_msg(send_ev.clone(), 3 * 60 * 60 * 1000, MainMessage::CheckElection));

	while let Some(main_message) = recieve_ev.next().await {
		match main_message {
			MainMessage::Gateway(deserialised) => match deserialised {
				GatewayRecieve::Dispatch { d, s } => {
					sequence_number = Some(s);

					debug!("received dispatch {:?}", d);
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
					if let Some(ping_squad) = bot_data.bills.get(&82) {
						for &subscriber in &ping_squad.subscribers {
							let Some(recipient_id) = bot_data.users.account_owner(subscriber) else {
								continue;
							};
							let embed = Embed::standard().with_title("Cheesebot Online").with_description(format!(
								"Cheesebot is now online. You received this message because you are subscribed to the {} bill.",
								&ping_squad.name
							));
							if let Err(e) = dm_embed(client, embed, recipient_id).await {
								error!("Failed to notify of cb online {e:?}");
							}
						}
					}

					bot_data.last_day = day;
					treasury_balance(bot_data, client).await;
					twaddle(bot_data, client).await;
					check_bills(bot_data, client).await;
				}
			}
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
