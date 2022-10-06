use std::collections::HashMap;

use chrono::Datelike;
use discord::*;
use serde::{Deserialize, Serialize};

pub type AccountId = u64;
pub const TREASURY: AccountId = 0;

/// The information tied to a specific discord userid
#[derive(Debug, Serialize, Deserialize)]
pub struct CheeseUser {
	pub account: AccountId,
	pub last_pay: chrono::DateTime<chrono::Utc>,
	pub organisations: Vec<AccountId>,
}

/// Data about an accout (organisation or personal)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Account {
	pub name: String,
	pub balance: u32,
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
	pub last_day: i32,
	pub treasury_balances: Vec<u32>,
	pub wealth_taxes: Vec<u32>,
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
			last_day: chrono::Utc::now().num_days_from_ce(),
			treasury_balances: Vec::new(),
			wealth_taxes: Vec::new(),
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

	pub fn treasury_account(&self) -> &Account {
		self.organisation_accounts.get(&TREASURY).unwrap()
	}

	pub fn treasury_account_mut(&mut self) -> &mut Account {
		self.organisation_accounts.get_mut(&TREASURY).unwrap()
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
	pub client: &'a mut DiscordClient,
	pub bot_data: &'a mut BotData,
	pub interaction: Interaction,
	pub user: User,
	pub options: HashMap<String, OptionType>,
}
