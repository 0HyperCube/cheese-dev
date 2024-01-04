use std::collections::HashMap;

use discord::*;

use crate::bot_data::*;

/// Utility function for responding to an interaction with a message
pub async fn respond_with_message<'a>(handler_data: &mut HandlerData<'a>, message: ChannelMessage) {
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
pub async fn respond_with_embed<'a>(handler_data: &mut HandlerData<'a>, embed: Embed) {
	respond_with_message(handler_data, ChannelMessage::new().with_embeds(embed)).await;
}

pub async fn respond_with_disappear_embed<'a>(handler_data: &mut HandlerData<'a>, embed: Embed) {
	respond_with_message(handler_data, ChannelMessage::new().with_embeds(embed).with_flags(1_u32 << 6)).await;
}

/// Utility function for dming a discord user a message
pub async fn dm_message<'a>(client: &mut DiscordClient, message: ChannelMessage, recipient_id: String) -> Result<(), NetError> {
	// We first create the channel (does nothing if it already exists)
	let channel = CreateDM { recipient_id }.post_create(client).await?;

	// Then we can send the message in the channel
	message.post_create(client, channel.id).await?;

	Ok(())
}

/// Utility function for dming a discord user an embed
pub async fn dm_embed<'a>(client: &mut DiscordClient, embed: Embed, recipient_id: String) -> Result<(), NetError> {
	dm_message(client, ChannelMessage::new().with_embeds(embed), recipient_id).await
}

/// Utility function to extract an account from a slash command option
pub async fn account_option<'a, V>(bot_data: &mut BotData, option: &OptionType, validation: V, user: &User) -> Option<u64>
where
	V: Fn(&BotData, AccountId, &User) -> bool,
{
	let parse_int = str::parse::<AccountId>(&option.as_str());
	match parse_int.map(|id| (id, validation(bot_data, id, user))) {
		Ok((id, true)) => Some(id),
		_ => None,
	}
}

/// Utility function for formating cheesecoin as `4.23cc`
pub fn format_cheesecoin(cc: u32) -> String {
	format!("{:.2}cc", cc as f64 / 100.)
}

pub struct ConstructedData<'a> {
	pub command: String,
	pub focused: Option<InteractionDataOption>,
	pub handler_data: HandlerData<'a>,
}

/// Construct the data handler (for implementing commands) from the specified interaction
///
/// This creates a new account if necessary, as well as flattenning subcommands into a space seperated string and finding the focused field
pub fn construct_handler_data<'a>(mut interaction: Interaction, client: &'a mut DiscordClient, bot_data: &'a mut BotData) -> ConstructedData<'a> {
	// Extract the user from the interaction (if in guild, then interaction["member"]["user"], if in dms then interaction["user"])
	let user = (interaction)
		.user
		.as_ref()
		.unwrap_or_else(|| &interaction.member.as_ref().unwrap().user)
		.clone();

	// If the user does not already have an account, create a new one.
	bot_data.cheese_user_mut(&user);

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

/// Handles transactions between accounts - returns (payer message, reciever message)
pub fn transact<'a>(handler_data: &mut HandlerData<'a>, recipiant: u64, from: u64, amount: f64) -> (String, Option<String>) {
	// Special error for negitive
	if amount < 0. {
		return ("Cannot pay a negative amount.".into(), None);
	}
	// Amount cast into real units
	let amount = (amount * 100.) as u32;
	if !handler_data.bot_data.accounts.exists(recipiant) {
		return (format!("To account does not exist"), None);
	}
	let Some(from) = handler_data.bot_data.accounts.account_mut(from) else {
		return (format!("From account does not exist"), None);
	};

	// Check the account can back the transaction
	if from.balance < amount {
		return (format!("{} has only {}.", from.name, format_cheesecoin(from.balance)), None);
	}
	from.balance -= amount;
	let payer_name = from.name.clone();

	let recipiant = handler_data.bot_data.accounts.account_mut(recipiant).unwrap();
	recipiant.balance += amount;

	let reciever_message = format!(
		"Your account - {} - has received {} from {}.",
		recipiant.name,
		format_cheesecoin(amount),
		payer_name
	);

	let sender_message = format!(
		"Successfully transfered {} from {} to {}.",
		format_cheesecoin(amount),
		payer_name,
		recipiant.name
	);

	(sender_message, Some(reciever_message))
}

pub fn format_bill(bill: &Bill, account_name: String) -> String {
	format!(
		"{} - {} to {} every {}{}",
		bill.name,
		format_cheesecoin(bill.amount),
		account_name,
		if bill.interval == 1 { "".to_string() } else { bill.interval.to_string() },
		if bill.interval == 1 { "day" } else { " days" }
	)
}
