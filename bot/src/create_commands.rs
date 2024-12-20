use discord::*;

pub async fn create_commands(client: &mut DiscordClient, application_id: &String) {
	let about = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("about")
		.with_description("Description of the bot.");
	let balances = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("balances")
		.with_description("All of your balances.");
	let pay = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("pay")
		.with_description("Give someone cheesecoins.")
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::String)
				.with_name("recipient")
				.with_description("recipient of the payment")
				.with_required(true)
				.with_autocomplete(true),
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
				.with_name("from")
				.with_description("The account the cheesecoins are from")
				.with_required(true)
				.with_autocomplete(true),
		);
	let organisation = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("organisation")
		.with_description("Organisation commands")
		.with_options(
			ApplicationCommandOption::new()
				.with_name("create")
				.with_description("Create an organisation.")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the new organisation"),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("transfer")
				.with_description("Transfer an organisation")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the organisation")
						.with_autocomplete(true),
				)
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("owner")
						.with_required(true)
						.with_description("The new owner of the organisation")
						.with_autocomplete(true),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("rename")
				.with_description("Rename an organisation")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the organisation")
						.with_autocomplete(true),
				)
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("new")
						.with_required(true)
						.with_description("The new name of the organisation"),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("delete")
				.with_description("Delete an organisation")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the organisation")
						.with_autocomplete(true),
				),
		);
	let bills = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("bill")
		.with_description("Bill commands")
		.with_options(
			ApplicationCommandOption::new()
				.with_name("create")
				.with_description("Create a bill.")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the new bill"),
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
						.with_name("to")
						.with_description("The account the cheesecoins go to")
						.with_required(true)
						.with_autocomplete(true),
				)
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::Number)
						.with_name("days")
						.with_description("Days between payments")
						.with_required(true),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("delete")
				.with_description("Delete a bill")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the bill")
						.with_autocomplete(true),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("subscribe")
				.with_description("Subscribe to a bill")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the bill")
						.with_autocomplete(true),
				)
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("from")
						.with_description("The account the cheesecoins are paid from")
						.with_required(true)
						.with_autocomplete(true),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("unsubscribe")
				.with_description("Unsubscribe from a bill")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::String)
						.with_name("name")
						.with_required(true)
						.with_description("The name of the bill")
						.with_autocomplete(true),
				),
		)
		.with_options(ApplicationCommandOption::new().with_name("view").with_description("View active bills"));
	let rollcall = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("claim")
		.with_description("Claim commands")
		.with_options(
			ApplicationCommandOption::new()
				.with_name("rollcall")
				.with_description("Claim your daily citizen rollcall"),
		);

	let party_option = ApplicationCommandOption::new()
		.with_option_type(CommandOptionType::String)
		.with_name("party")
		.with_required(true)
		.with_description("The party")
		.with_autocomplete(true);

	let parliament = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("parliament")
		.with_description("Parliament commands")
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::SubCommandGroup)
				.with_name("add")
				.with_description("Add something.")
				.with_options(
					ApplicationCommandOption::new()
						.with_name("party")
						.with_description("Add a party to the election.")
						.with_options(party_option.clone().with_autocomplete(false)),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::SubCommandGroup)
				.with_name("delete")
				.with_description("Delete something.")
				.with_options(
					ApplicationCommandOption::new()
						.with_name("party")
						.with_description("Delete a party from the election.")
						.with_options(party_option.clone()),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("vote")
				.with_description("Vote for a candidate (or change your vote).")
				.with_options(party_option.clone()),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::SubCommandGroup)
				.with_name("view")
				.with_description("View something.")
				.with_options(
					ApplicationCommandOption::new()
						.with_name("results")
						.with_description("View results of last election."),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::SubCommandGroup)
				.with_name("count")
				.with_description("Count something.")
				.with_options(
					ApplicationCommandOption::new()
						.with_name("results")
						.with_description("Count results of last election."),
				),
		);

	let role = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("role")
		.with_description("Cosmetic role options (15cc)")
		.with_options(
			ApplicationCommandOption::new()
				.with_name("assign")
				.with_description("Assign yourself a cosmetic role for a fee.")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::Number)
						.with_name("r")
						.with_description("Red (0-255)")
						.with_required(true),
				)
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::Number)
						.with_name("g")
						.with_description("Green (0-255)")
						.with_required(true),
				)
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::Number)
						.with_name("b")
						.with_description("Blue (0-255)")
						.with_required(true),
				),
		);
	let decree = ApplicationCommand::new()
		.with_name("decree")
		.with_description("Decree (si tu es président)")
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::String)
				.with_name("title")
				.with_description("Title of decree")
				.with_required(true),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::String)
				.with_name("description")
				.with_description("Description of decree")
				.with_required(true),
		);

	let sudo = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("sudo")
		.with_description("super user do")
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::SubCommandGroup)
				.with_name("print")
				.with_description("Utilise a printer.")
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::SubCommand)
						.with_name("cheesecoin")
						.with_description("Print cheesecoin with a printer.")
						.with_options(
							ApplicationCommandOption::new()
								.with_option_type(CommandOptionType::String)
								.with_name("recipient")
								.with_description("recipient of the payment")
								.with_required(true)
								.with_autocomplete(true),
						)
						.with_options(
							ApplicationCommandOption::new()
								.with_option_type(CommandOptionType::Number)
								.with_name("cheesecoin")
								.with_description("Number of cheesecoin")
								.with_required(true),
						),
				),
		);

	ApplicationCommandList::new()
		.with_commands(about)
		.with_commands(balances)
		.with_commands(pay)
		.with_commands(bills)
		.with_commands(rollcall)
		.with_commands(organisation)
		.with_commands(parliament)
		.with_commands(role)
		.with_commands(decree)
		.with_commands(sudo)
		.put_bulk_override_global(client, application_id)
		.await
		.unwrap();
}
