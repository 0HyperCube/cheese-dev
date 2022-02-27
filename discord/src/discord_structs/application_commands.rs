use super::prelude::*;
use super::ChannelMessage;
use super::MessageComponent;

#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Default)]
#[repr(u8)]
pub enum CommandType {
	#[default]
	Chat = 1,
	User = 2,
	Message = 3,
}
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Default)]
#[repr(u8)]
pub enum InteractionType {
	#[default]
	Ping = 1,
	ApplicationCommand = 2,
	MessageComponent = 3,
	ApplicationCommandAutocomplete = 4,
	ModalSubmit = 5,
}
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Default)]
#[repr(u8)]
pub enum CommandOptionType {
	#[default]
	SubCommand = 1,
	SubCommandGroup = 2,
	String = 3,
	Integer = 4,
	Bool = 5,
	User = 6,
	Channel = 7,
	Role = 8,
	Mentionable = 9,
	Number = 10,
	Attachment = 11,
}
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(untagged)]
pub enum OptionType {
	#[default]
	None,
	String(String),
	Integer(u64),
	Number(f64),
}

#[discord_struct]
pub struct Modal {
	custom_id: String,
	title: String,
	components: Vec<MessageComponent>,
}

#[discord_struct]
pub struct ApplicationCommandOptionChoice {
	name: String,
	value: OptionType,
}
#[discord_struct]
pub struct ApplicationCommandOption {
	#[serde(rename = "type")]
	option_type: CommandOptionType,
	name: String,
	description: String,
	required: bool,
	autocomplete: bool,
	choices: Option<Vec<ApplicationCommandOptionChoice>>,
	options: Option<Vec<ApplicationCommandOption>>,
}

#[request(add_command = POST "/applications/{application_id}/commands" as application_id)]
#[discord_struct]
pub struct ApplicationCommand {
	#[serde(rename = "type")]
	command_type: Option<CommandType>,
	name: String,

	description: Option<String>,
	options: Option<Vec<ApplicationCommandOption>>,
}

#[request(bulk_override_global as {&self.commands} = PUT "/applications/{application_id}/commands" as application_id)]
#[discord_struct]
pub struct ApplicationCommandList {
	commands: Vec<ApplicationCommand>,
}

#[discord_struct]
pub struct InteractionData {
	id: String,
	name: String,
	#[serde(rename = "type")]
	command_type: CommandType,
	components: Option<Vec<MessageComponent>>,
}

/// https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-structure
#[discord_struct]
pub struct Interaction {
	id: String,
	application_id: String,
	#[serde(rename = "type")]
	interaction_type: InteractionType,
	data: Option<InteractionData>,
	channel_id: String,
	token: String,
}

#[derive(Clone, Debug)]
#[serialise_tag("type")]
pub enum InteractionResponse {
	#[tag(1)]
	Pong,
	#[tag(4)]
	ChannelMessageWithSource { data: ChannelMessage },
	#[tag(5)]
	DeferredChannelMessageWithSource,
	#[tag(7)]
	UpdateMessage,
	#[tag(8)]
	ApplicationCommandAutocompleteResult,
	#[tag(9)]
	Modal { data: Modal },
}

#[request(respond as {&self.value} = POST "/interactions/{interaction_id}/{interaction_token}/callback" as interaction_id, interaction_token)]
pub struct InteractionCallback {
	value: InteractionResponse,
}

impl InteractionCallback {
	pub fn new(value: InteractionResponse) -> Self {
		Self { value }
	}
}
