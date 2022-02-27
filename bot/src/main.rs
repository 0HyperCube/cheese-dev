use std::{
	collections::HashMap,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc,
	},
};

use discord::{async_channel::Sender, *};

#[macro_use]
extern crate log;

pub struct BotData {
	pub names: Vec<String>,
}

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

async fn send_heartbeat(send_outgoing_message: &Sender<String>, sequence_number: &Arc<AtomicUsize>) {
	let val = sequence_number.load(Ordering::SeqCst);
	let heartbeat = if val == usize::MAX { None } else { Some(val) };

	send_outgoing_message
		.send(serde_json::to_string(&GatewaySend::Heartbeat { d: heartbeat }).unwrap())
		.await
		.unwrap();
}

/// Sends a heartbeat every `period` milliseconds
async fn heartbeat(send_outgoing_message: Sender<String>, sequence_number: Arc<AtomicUsize>, interval: u64) {
	let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(interval));
	loop {
		interval.tick().await;
		send_heartbeat(&send_outgoing_message, &sequence_number).await;
	}
}

fn construct_handler_data<'a>(
	mut interaction: Interaction,
	client: &'a mut DiscordClient,
	bot_data: &'a mut BotData,
) -> (String, Option<InteractionDataOption>, HandlerData<'a>) {
	let user = (interaction)
		.user
		.as_ref()
		.unwrap_or_else(|| &interaction.member.as_ref().unwrap().user)
		.clone();
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

async fn respond_with_embed<'a>(handler_data: HandlerData<'a>, embed: Embed) {
	InteractionCallback::new(InteractionResponse::ChannelMessageWithSource {
		data: ChannelMessage::new().with_embeds(embed),
	})
	.post_respond(handler_data.client, handler_data.interaction.id, handler_data.interaction.token)
	.await
	.unwrap();
}

async fn about<'a>(handler_data: HandlerData<'a>) {
	respond_with_embed(
		handler_data,
		Embed::standard()
			.with_title("About")
			.with_description("This bot is developed by Go Consulting Ltd. to handle the finances of New New Cheeseland."),
	)
	.await;
}

async fn balances<'a>(handler_data: HandlerData<'a>) {
	respond_with_embed(handler_data, Embed::standard().with_title("Balances")).await;
}

async fn pay<'a>(handler_data: HandlerData<'a>) {
	respond_with_embed(handler_data, Embed::standard().with_title("Pay")).await;
}

async fn organisation_create<'a>(handler_data: HandlerData<'a>) {
	let name = handler_data.options["name"].as_str();
	respond_with_embed(handler_data, Embed::standard().with_title("Create Organisation").with_description(name)).await;
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

			let choices = handler_data
				.bot_data
				.names
				.iter()
				.filter(|name| name.to_lowercase().contains(&str_value))
				.enumerate()
				.filter(|(index, _)| *index < 25) // Discord does not allow >25 options.
				.map(|(_, value)| value)
				.map(|name| {
					ApplicationCommandOptionChoice::new()
						.with_name(name)
						.with_value(OptionType::String(name.clone()))
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

/// Runs the bot
async fn run() {
	let mut client = DiscordClient::new(include_str!("token.txt"));

	let mut bot_data = BotData {
		names: vec![
			"a".to_string(),
			"b".to_string(),
			"c".to_string(),
			"d".to_string(),
			"e".to_string(),
			"f".to_string(),
			"g".to_string(),
			"h".to_string(),
			"i".to_string(),
			"j".to_string(),
			"k".to_string(),
			"l".to_string(),
			"m".to_string(),
			"n".to_string(),
			"o".to_string(),
			"p".to_string(),
			"q".to_string(),
			"r".to_string(),
			"s".to_string(),
			"t".to_string(),
			"u".to_string(),
			"v".to_string(),
			"w".to_string(),
			"x".to_string(),
			"y".to_string(),
			"z".to_string(),
			"Bob".to_string(),
			"Jeff".to_string(),
		],
	};

	let gateway = GatewayMeta::get_gateway_meta(&mut client).await.unwrap();
	info!("Recieved gateway metadata: {:?}", gateway);

	let Connection {
		send_outgoing_message,
		mut read,
		sequence_number,
	} = client.connect_gateway(gateway.url).await;

	while let Some(Ok(Message::Text(text))) = read.next().await {
		match serde_json::from_str(&text) {
			Ok(deserialised) => match deserialised {
				GatewayRecieve::Dispatch { d, s } => {
					sequence_number.store(s, Ordering::SeqCst);

					debug!("Recieved dispatch {:?}", d);
					match d {
						Dispatch::Ready(r) => create_commands(&mut client, &r.application.id).await,
						Dispatch::InteractionCreate(interaction) => handle_interaction(interaction, &mut client, &mut bot_data).await,
						_ => warn!("Unhandled dispatch"),
					}
				}
				GatewayRecieve::Heartbeat { .. } => {
					warn!("Discord wants a heartbeat, sending");
					send_heartbeat(&send_outgoing_message, &sequence_number).await;
				}
				GatewayRecieve::Reconnect => todo!(),
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
					tokio::spawn(heartbeat(send_outgoing_message.clone(), sequence_number.clone(), d.heartbeat_interval));
				}
				GatewayRecieve::HeartbeatACK => {}
			},
			Err(e) => {
				error!("Error decoding gateway message {:?}", e);
				debug!("Gateway message {}", text);
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
						.with_required(true),
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
		.block_on(run());
}
