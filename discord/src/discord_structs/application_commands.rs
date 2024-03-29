use core::panic;

use crate::GuildMember;
use crate::User;

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
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Default, PartialEq, Eq)]
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

impl OptionType {
	pub fn as_str(&self) -> String {
		match self {
			OptionType::String(x) => x.to_string(),
			_ => unimplemented!("Invalid type for str"),
		}
	}
	pub fn as_float(&self) -> f64 {
		match self {
			OptionType::Number(x) => *x,
			_ => unimplemented!("Invalid type for float"),
		}
	}
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
pub struct InteractionDataOption {
	name: String,
	#[serde(rename = "type")]
	option_type: CommandOptionType,
	value: Option<OptionType>,
	options: Option<Vec<InteractionDataOption>>,
	focused: Option<bool>,
}

#[discord_struct]
pub struct InteractionData {
	id: Option<String>,
	name: Option<String>,
	#[serde(rename = "type")]
	command_type: Option<CommandType>,
	components: Option<Vec<MessageComponent>>,
	options: Option<Vec<InteractionDataOption>>,

	custom_id: Option<String>,
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
	member: Option<GuildMember>,
	user: Option<User>,
}

#[test]
fn test_de() {
	let x = serde_json::from_str::<Interaction>(
		r##"{"version":1,"user":{"username":"Käse","public_flags":128,"id":"630073509137350690","discriminator":"3615","avatar_decoration":null,"avatar":"761226738b7c90394e301b7f387fdc9d"},"type":3,"token":"aW50ZXJhY3Rpb246OTcwMDc1Mjc5ODIzMzYwMDEwOklTSFo2Tlpjb1BpUGx2V3NQcUhUT241cEs2a3czVFV4b2V5QnJzS3MzY2pmSmp3Q2JnM0pHaTlma0U4cGJ5dEVjSEtyRElINnR2aDdkczBCS0JzTndKVWszOGNSTEd2aEgzQVljaXZ6bEFNd1ZFaEZLRWZ0QmRyV3BNMHY3cVBm","message":{"webhook_id":"910254320740610069","type":20,"tts":false,"timestamp":"2022-04-30T21:32:08.028000+00:00","pinned":false,"mentions":[],"mention_roles":[],"mention_everyone":false,"interaction":{"user":{"username":"Käse","public_flags":128,"id":"630073509137350690","discriminator":"3615","avatar_decoration":null,"avatar":"761226738b7c90394e301b7f387fdc9d"},"type":2,"name":"parliament","id":"970075104019099648"},"id":"970075104463683645","flags":0,"embeds":[],"edited_timestamp":null,"content":"Vote for your candidate:","components":[{"type":1,"components":[{"type":2,"style":2,"label":"James","custom_id":"630073509137350690"}]}],"channel_id":"910597009466093628","author":{"username":"Cheese Bot","public_flags":0,"id":"910254320740610069","discriminator":"4538","bot":true,"avatar_decoration":null,"avatar":null},"attachments":[],"application_id":"910254320740610069"},"locale":"en-GB","id":"970075279823360010","data":{"custom_id":"630073509137350690","component_type":2},"channel_id":"910597009466093628","application_id":"910254320740610069"}"##,
	);
	x.unwrap();
	let y: Interaction = serde_json::from_str(r##"
{"version":1,"type":3,"token":"aW50ZXJhY3Rpb246MTA3ODA2MzY2OTg3Njg4NzY1Mjo2RDRLVUg2c1FvSTlSa2w0Q2VkR3NMelhiN2pXMUpDMzNWT2VMeTlDZzEyZWUyWWRNc2JWd1psdWJsOE5GZzBCTGZnYUM0OXEzZ1E1eHJxZFBEb3ZDY1o5RTlBa2RHNzJyUGZ0ejVaeHd5R1dpQ0lXYVNpWFVuUVBoaEo5eDY2NA","message":{"webhook_id":"910254320740610069","type":20,"tts":false,"timestamp":"2023-02-22T21:11:38.400000+00:00","pinned":false,"mentions":[],"mention_roles":[],"mention_everyone":false,"interaction":{"user":{"username":"Käse","public_flags":128,"id":"630073509137350690","display_name":null,"discriminator":"3615","avatar_decoration":null,"avatar":"761226738b7c90394e301b7f387fdc9d"},"type":2,"name":"parliament vote","id":"1078061529682956439"},"id":"1078061530978992240","flags":0,"embeds":[],"edited_timestamp":null,"content":"Vote for your candidate:","components":[{"type":1,"components":[{"type":2,"style":2,"label":"Alan","custom_id":"vote887020108696920116"},{"type":2,"style":2,"label":"аe","custom_id":"vote762325231925854231"},{"type":2,"style":2,"label":"Elliot F.","custom_id":"vote722468356711776269"}]}],"channel_id":"956233767767408741","author":{"username":"Cheese Bot","public_flags":0,"id":"910254320740610069","display_name":null,"discriminator":"4538","bot":true,"avatar_decoration":null,"avatar":null},"attachments":[],"application_id":"910254320740610069"},"member":{"user":{"username":"18XYang","public_flags":0,"id":"764206694841974795","display_name":null,"discriminator":"6684","avatar_decoration":null,"avatar":null},"roles":["1078035383511699496","985630968650010705","985804444237172797"],"premium_since":null,"permissions":"539018919105","pending":false,"nick":"Xiao-Kun","mute":false,"joined_at":"2023-02-22T19:00:46.675000+00:00","is_pending":false,"flags":0,"deaf":false,"communication_disabled_until":null,"avatar":null},"locale":"en-GB","id":"1078063669876887652","guild_locale":"en-US","guild_id":"907657508292792342","entitlement_sku_ids":[],"data":{"custom_id":"vote887020108696920116","component_type":2},"channel_id":"956233767767408741","application_id":"910254320740610069","app_permissions":"4398046511103"}"##).unwrap();
}

#[discord_struct]
pub struct AutocompleteResult {
	choices: Vec<ApplicationCommandOptionChoice>,
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
	ApplicationCommandAutocompleteResult { data: AutocompleteResult },
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
