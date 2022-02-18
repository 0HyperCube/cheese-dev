use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Arc,
};

use discord::{async_channel::Sender, *};

#[macro_use]
extern crate log;

// Use simplelog with a file and the console.
fn init_logger() {
	use simplelog::*;
	use std::fs::File;

	CombinedLogger::init(vec![
		TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
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

/// Runs the bot
async fn run() {
	let mut client = DiscordClient::new(include_str!("token.txt"));

	let gateway = GatewayMeta::get_gateway_meta(&mut client).await.unwrap();
	info!("Recieved gateway metadata: {:?}", gateway);

	let Connection {
		send_outgoing_message,
		mut read,
		sequence_number,
	} = client.connect_gateway(gateway.url).await;

	let mut application_id;

	while let Some(Ok(Message::Text(text))) = read.next().await {
		match serde_json::from_str(&text) {
			Ok(deserialised) => {
				info!("{:?}", deserialised);

				match deserialised {
					GatewayRecieve::Dispatch { d, s } => {
						sequence_number.store(s, Ordering::SeqCst);

						info!("Recieved dispatch {:?}", d);
						match d {
							Dispatch::Ready(r) => {
								application_id = r.application.id;

								ApplicationCommandList::new()
									.with_commands(
										ApplicationCommand::new()
											.with_command_type(CommandType::Chat)
											.with_name("bob")
											.with_description("Responds with embed"),
									)
									.with_commands(
										ApplicationCommand::new()
											.with_command_type(CommandType::Chat)
											.with_name("jeff")
											.with_description("Responds with modal"),
									)
									.put_bulk_override_global(&mut client, &application_id)
									.await
									.unwrap();
							}
							Dispatch::InteractionCreate(interaction) => {
								// InteractionCallback::new(InteractionResponse::ChannelMessageWithSource {
								// 	data: ChannelMessage::new().with_content("Hello").with_components(
								// 		ActionRows::new()
								// 			.with_components(Button::new().with_custom_id("yyy").with_label("yyy")),
								// 	),
								// })
								// .post_respond(&mut client, interaction.id, interaction.token)
								// .await
								// .unwrap();
								InteractionCallback::new(InteractionResponse::Modal {
									data: Modal::new().with_custom_id("pay").with_title("Pay").with_components(
										ActionRows::new().with_components(
											TextInput::new()
												.with_custom_id("amount")
												.with_label("Amount")
												.with_style(TextInputStyle::Short)
												.with_min_length(1)
												.with_max_length(6)
												.with_placeholder("In format {}{}.{}{}"),
										),
									),
								})
								.post_respond(&mut client, interaction.id, interaction.token)
								.await
								.unwrap();
							}
							_ => {
								warn!("Unhandled dispatch")
							}
						}
					}
					GatewayRecieve::Heartbeat { .. } => {
						warn!("Discord wants a heartbeat, sending");
						send_heartbeat(&send_outgoing_message, &sequence_number).await;
					}
					GatewayRecieve::Reconnect => todo!(),
					GatewayRecieve::InvalidSession { d } => {
						error!("Invalid session, can reconnect {}", d);
					}
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
				}
			}
			Err(e) => {
				error!("Error decoding gateway message {}: {:?}", text, e);
			}
		}
	}
}

fn main() {
	init_logger();

	tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(run());
}
