use super::prelude::*;

#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Default)]
#[repr(u8)]
pub enum ComponentType {
	#[default]
	ActionRow = 1,
	Button = 2,
	SelectMenu = 3,
	TextInput = 4,
}
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Default)]
#[repr(u8)]
pub enum ButtonStyle {
	#[default]
	Primary = 1,
	Secondary = 2,
	Success = 3,
	Danger = 4,
	Link = 5,
}
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, Default)]
#[repr(u8)]
pub enum TextInputStyle {
	#[default]
	Short = 1,
	Paragraph = 2,
}

#[discord_struct]
pub struct ActionRows {
	components: Vec<MessageComponent>,
}

#[discord_struct]
pub struct Button {
	custom_id: Option<String>,

	style: Option<ButtonStyle>,
	label: Option<String>,
	url: Option<String>,
	disabled: Option<bool>,
}
#[discord_struct]
pub struct SelectOption {
	label: String,
	value: String,
	description: Option<String>,
	default: Option<bool>,
}
#[discord_struct]
pub struct SelectMenu {
	custom_id: Option<String>,

	options: Vec<SelectOption>,
	placeholder: Option<String>,
	disabled: Option<bool>,
}
#[discord_struct]
pub struct TextInput {
	custom_id: Option<String>,

	style: Option<TextInputStyle>,
	label: Option<String>,
	value: Option<String>,
	placeholder: Option<String>,
	min_length: Option<i32>,
	max_length: Option<i32>,
}

#[derive(Clone, Debug)]
#[serialise_tag("type")]
pub enum MessageComponent {
	#[tag(1)]
	ActionRows(ActionRows),
	#[tag(2)]
	Button(Button),
	#[tag(3)]
	SelectMenu(SelectMenu),
	#[tag(4)]
	TextInput(TextInput),
}
impl From<ActionRows> for MessageComponent {
	fn from(action_rows: ActionRows) -> Self {
		Self::ActionRows(action_rows)
	}
}
impl From<Button> for MessageComponent {
	fn from(button: Button) -> Self {
		Self::Button(button)
	}
}
impl From<SelectMenu> for MessageComponent {
	fn from(select_menu: SelectMenu) -> Self {
		Self::SelectMenu(select_menu)
	}
}
impl From<TextInput> for MessageComponent {
	fn from(text_input: TextInput) -> Self {
		Self::TextInput(text_input)
	}
}
