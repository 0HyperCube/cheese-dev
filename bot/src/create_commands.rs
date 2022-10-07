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
				.with_name("recipiant")
				.with_description("Recipiant of the payment")
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
				.with_description("Claim your MP daily rollcall"),
		);
	let parliament = ApplicationCommand::new()
		.with_command_type(CommandType::Chat)
		.with_name("parliament")
		.with_description("Parliament commands")
		.with_options(ApplicationCommandOption::new().with_name("run").with_description("Run as candidate."))
		.with_options(
			ApplicationCommandOption::new()
				.with_option_type(CommandOptionType::SubCommandGroup)
				.with_name("stop")
				.with_description("Stop doing something.")
				.with_options(
					ApplicationCommandOption::new()
						.with_name("running")
						.with_description("Stop running as candidate"),
				),
		)
		.with_options(
			ApplicationCommandOption::new()
				.with_name("vote")
				.with_description("Vote for a candidate (or change your vote)."),
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
		);
	ApplicationCommandList::new()
		.with_commands(about)
		.with_commands(balances)
		.with_commands(pay)
		.with_commands(rollcall)
		.with_commands(organisation)
		.with_commands(parliament)
		.put_bulk_override_global(client, application_id)
		.await
		.unwrap();
}
