use std::collections::HashMap;

use crate::CheeseCoinTy;
use chrono::Datelike;
use discord::*;
use serde::{Deserialize, Serialize};
pub use std::fmt::Write;

use crate::format_bill;

pub type AccountId = u64;
pub const TREASURY: AccountId = 0;

/// The information tied to a specific discord userid
#[derive(Debug, Serialize, Deserialize)]
pub struct CheeseUser {
	pub account: AccountId,
	pub last_pay: chrono::DateTime<chrono::Utc>,
	pub organisations: Vec<AccountId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub role_id: Option<String>,
}

/// A bill which has been created by a particular account
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Bill {
	pub name: String,
	pub last_pay: i32,
	pub interval: i32,
	pub amount: CheeseCoinTy,
	pub owner: AccountId,
	pub subscribers: Vec<AccountId>,
}
pub type BillId = u64;

/// Data about an accout (organisation or personal)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Account {
	pub name: String,
	pub balance: CheeseCoinTy,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub owned_bills: Vec<BillId>,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub subscribed_bills: Vec<BillId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Accounts {
	pub personal_accounts: HashMap<AccountId, Account>,
	pub organisation_accounts: HashMap<AccountId, Account>,
}

impl Accounts {
	/// Get the account from an account id (either personal or organisation)
	pub fn account_mut(&mut self, account: AccountId) -> Option<&mut Account> {
		self.personal_accounts
			.get_mut(&account)
			.map_or_else(|| self.organisation_accounts.get_mut(&account), |x| Some(x))
	}

	/// Get the account from an account id (either personal or organisation)
	pub fn account(&self, account: AccountId) -> Option<&Account> {
		self.personal_accounts
			.get(&account)
			.map_or_else(|| self.organisation_accounts.get(&account), |x| Some(x))
	}

	pub fn exists(&self, account: AccountId) -> bool {
		self.account(account).is_some()
	}
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Users {
	pub users: HashMap<String, CheeseUser>,
}

impl Users {
	pub fn get_mut(&mut self, user: &String) -> Option<&mut CheeseUser> {
		self.users.get_mut(user)
	}
	/// Finds the account owner from an account id
	pub fn account_owner(&self, account: AccountId) -> Option<String> {
		self.users
			.iter()
			.find(|(_, user)| user.account == account || user.organisations.contains(&account))
			.map(|(name, _)| name.clone())
	}
}

/// All the data the bot saves
#[derive(Debug, Serialize, Deserialize)]
pub struct BotData {
	pub users: Users,
	pub accounts: Accounts,
	#[serde(default)]
	pub bills: HashMap<BillId, Bill>,
	pub next_account: AccountId,
	pub wealth_tax: Vec<(CheeseCoinTy, f64)>,
	pub vat: f64,
	pub last_wealth_tax: chrono::DateTime<chrono::Utc>,
	pub last_day: i32,
	pub treasury_balances: Vec<CheeseCoinTy>,
	pub wealth_taxes: Vec<CheeseCoinTy>,
	#[serde(default)]
	pub parties: HashMap<String, Vec<String>>,
	pub previous_time: chrono::DateTime<chrono::Utc>,
	pub previous_results: String,
	#[serde(skip)]
	pub file_path: String,
	#[serde(default)]
	pub decree: u64,
}

impl Default for BotData {
	fn default() -> Self {
		let organisation_accounts = HashMap::from([(
			0,
			Account {
				name: "Treasury".into(),
				balance: 1000,
				..Default::default()
			},
		)]);
		Self {
			users: Users { users: HashMap::new() },
			accounts: Accounts {
				personal_accounts: HashMap::new(),
				organisation_accounts,
			},
			bills: HashMap::new(),
			next_account: 1,
			wealth_tax: vec![(1000, 5.), (10000, 7.), (CheeseCoinTy::MAX, 10.)],
			vat: 2.,
			last_wealth_tax: chrono::Utc::now(),
			last_day: chrono::Utc::now().num_days_from_ce(),
			treasury_balances: Vec::new(),
			wealth_taxes: Vec::new(),
			parties: HashMap::new(),
			previous_time: chrono::Utc::now(),
			previous_results: "No previous results".into(),
			file_path: String::new(),
			decree: 0,
		}
	}
}

impl BotData {
	/// Get the cheese user information given a discord user
	pub fn cheese_user<'a>(&'a self, user: &User) -> &'a CheeseUser {
		&self.users.users[&user.id]
	}

	/// Get the cheese user information given a discord user
	pub fn cheese_user_mut<'a>(&'a mut self, user: &User) -> &'a mut CheeseUser {
		self.users.users.entry(user.id.clone()).or_insert_with(|| {
			self.accounts.personal_accounts.insert(
				self.next_account,
				Account {
					name: user.username.clone(),
					balance: 0,
					..Default::default()
				},
			);
			self.next_account += 1;
			CheeseUser {
				account: self.next_account - 1,
				last_pay: chrono::DateTime::<chrono::Utc>::MIN_UTC,
				organisations: Vec::new(),
				role_id: None,
			}
		})
	}

	/// Get the personal account name from a discord user
	pub fn personal_account_name(&self, user: &User) -> String {
		self.accounts.personal_accounts[&self.cheese_user(user).account].name.clone()
	}

	/// Checks if the given account id exists at all
	pub fn account_exists(&self, account: AccountId, _user: &User) -> bool {
		self.accounts.personal_accounts.contains_key(&account) || self.accounts.organisation_accounts.contains_key(&account)
	}

	/// Checks if the given personal account id exists at all
	pub fn personal_account_exists(&self, account: AccountId, _user: &User) -> bool {
		self.accounts.personal_accounts.contains_key(&account)
	}

	/// Checks if the given account id is owned by the specified user (personal or owned organisation)
	pub fn account_owned(&self, account: AccountId, user: &User) -> bool {
		let cheese_user = self.cheese_user(user);
		account == cheese_user.account || cheese_user.organisations.contains(&account)
	}

	/// Checks if the given bill id exists at all
	pub fn bill_exists(&self, bill: AccountId, _user: &User) -> bool {
		self.bills.contains_key(&bill)
	}

	/// Checks if the given bill id is owned by the specified user (personal or owned organisation)
	pub fn bill_owned(&self, bill: BillId, user: &User) -> bool {
		let cheese_user = self.cheese_user(user);
		self.accounts
			.personal_accounts
			.get(&cheese_user.account)
			.filter(|acc| acc.owned_bills.contains(&bill))
			.is_some()
			|| cheese_user.organisations.iter().any(|org| {
				self.accounts
					.organisation_accounts
					.get(org)
					.filter(|acc| acc.owned_bills.contains(&bill))
					.is_some()
			})
	}

	/// Checks if the given bill id is subscribed by the specified user (personal or owned organisation)
	pub fn bill_subscribed(&self, bill: BillId, user: &User) -> bool {
		let cheese_user = self.cheese_user(user);
		self.accounts
			.personal_accounts
			.get(&cheese_user.account)
			.filter(|acc| acc.subscribed_bills.contains(&bill))
			.is_some()
			|| cheese_user.organisations.iter().any(|org| {
				self.accounts
					.organisation_accounts
					.get(org)
					.filter(|acc| acc.subscribed_bills.contains(&bill))
					.is_some()
			})
	}

	/// Computes the total currency in circulation (for currency information in balances)
	pub fn total_currency(&self) -> CheeseCoinTy {
		let personal = self.accounts.personal_accounts.iter().map(|(_, a)| a.balance);
		let orgs = self.accounts.organisation_accounts.iter().map(|(_, a)| a.balance);
		personal.chain(orgs).fold(0, |a, b| a.saturating_add(b))
	}

	pub fn treasury_account(&self) -> &Account {
		self.accounts.organisation_accounts.get(&TREASURY).unwrap()
	}

	pub fn treasury_account_mut(&mut self) -> &mut Account {
		self.accounts.organisation_accounts.get_mut(&TREASURY).unwrap()
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
		self.accounts
			.personal_accounts
			.iter()
			.map(|(id, account)| (account.name.clone() + self.option_suffix(id, " (Personal)"), *id))
	}
	/// List all people names (with added suffix) and ids
	pub fn people(&self) -> impl Iterator<Item = (String, AccountId)> + '_ {
		self.accounts
			.personal_accounts
			.iter()
			.map(|(id, account)| (account.name.clone() + self.option_suffix(id, " (Person)"), *id))
	}
	/// List all non-self people names (with added suffix) and ids
	pub fn non_self_people(&self, user: &User) -> impl Iterator<Item = (String, AccountId)> + '_ {
		let user = self.cheese_user(user);
		self.accounts
			.personal_accounts
			.iter()
			.filter(|(id, _)| **id != user.account)
			.map(|(id, account)| (account.name.clone() + self.option_suffix(id, " (Person)"), *id))
	}
	/// List all organisation account names (with added suffix) and ids
	pub fn organisation_accounts(&self) -> impl Iterator<Item = (String, AccountId)> + '_ {
		self.accounts
			.organisation_accounts
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
			.map(|org| (org, &self.accounts.organisation_accounts[org]))
			.map(|(id, account)| (account.name.clone() + self.option_suffix(id, " (Organisation)"), *id))
	}
	/// List the user's owned bills
	pub fn owned_bills(&self, user: &User) -> impl Iterator<Item = (String, BillId)> + '_ {
		let user = self.cheese_user(user);
		user.organisations
			.iter()
			.copied()
			.chain([user.account])
			.filter_map(|account_id| self.accounts.account(account_id))
			.flat_map(|account| account.owned_bills.iter().map(|bill| (bill, account.name.clone())))
			.filter_map(|(bill_id, account_name)| self.bills.get(&bill_id).map(|bill| (bill, account_name, bill_id)))
			.map(|(bill, account_name, &bill_id)| (format_bill(bill, account_name), bill_id))
	}
	/// List the user's subscribed bills
	pub fn subscribed_bills(&self, user: &User) -> impl Iterator<Item = (String, BillId)> + '_ {
		let user = self.cheese_user(user);
		user.organisations
			.iter()
			.copied()
			.chain([user.account])
			.filter_map(|account_id| self.accounts.account(account_id))
			.flat_map(|account| account.subscribed_bills.iter().map(|bill| (bill, account.name.clone())))
			.filter_map(|(bill_id, account_name)| self.bills.get(&bill_id).map(|bill| (bill, account_name, bill_id)))
			.map(|(bill, account_name, &bill_id)| (format_bill(bill, account_name), bill_id))
	}
	/// List the user's owned bills
	pub fn bills(&self) -> impl Iterator<Item = (String, BillId)> + '_ {
		self.bills
			.iter()
			.filter_map(|(bill_id, bill)| self.accounts.account(bill.owner).map(|account| (bill, account.name.clone(), bill_id)))
			.map(|(bill, account_name, &bill_id)| (format_bill(bill, account_name), bill_id))
	}

	/// List the parties
	pub fn parties(&self) -> impl Iterator<Item = String> + '_ {
		self.parties.keys().map(|v| v.clone())
	}

	pub fn save(&self) {
		let new = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new().indentor(String::from("\t"))).unwrap();
		std::fs::write(&self.file_path, new).unwrap();
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
