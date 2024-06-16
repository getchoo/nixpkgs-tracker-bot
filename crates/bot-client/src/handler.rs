use crate::{SharedConfig, SharedHttp};
use bot_error::Error;

use log::{debug, error, info, trace, warn};
use serenity::all::CreateBotAuthParameters;
use serenity::async_trait;
use serenity::builder::{
	CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup,
	CreateInteractionResponseMessage,
};
use serenity::model::{
	application::{Command, CommandInteraction, Interaction},
	colour::Colour,
	gateway::Ready,
};
use serenity::prelude::{Context, EventHandler};

#[derive(Clone, Copy, Debug)]
pub struct Handler;

impl Handler {
	async fn register_commands(&self, ctx: &Context) -> Result<(), Error> {
		let commands = bot_commands::to_vec();
		let commands_len = commands.len();
		for command in commands {
			Command::create_global_command(&ctx.http, command).await?;
		}

		debug!("Registered {} commands", commands_len);
		Ok(())
	}

	/// Dispatch our commands from a [`CommandInteraction`]
	async fn dispatch_command(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
		let command_name = command.data.name.as_str();

		// grab our configuration & http client from the aether
		let (http, config) = {
			let read = ctx.data.read().await;
			let http = read
				.get::<SharedHttp>()
				.ok_or("Couldn't get shared HTTP client! WHY??????")?
				.clone();
			let config = read
				.get::<SharedConfig>()
				.ok_or("Couldn't get shared bot configuration!")?
				.clone();
			(http, config)
		};

		match command_name {
			"ping" => bot_commands::ping::respond(ctx, command).await?,
			"track" => bot_commands::track::respond(ctx, &http, &config, command).await?,
			_ => {
				let message = CreateInteractionResponseMessage::new().content(format!(
					"It doesn't look like you can use `{command_name}`. Sorry :("
				));
				let response = CreateInteractionResponse::Message(message);
				command.create_response(&ctx, response).await?;
			}
		};

		Ok(())
	}

	async fn invite_link(ctx: &Context) {
		if let Ok(invite_link) = CreateBotAuthParameters::new().auto_client_id(ctx).await {
			let link = invite_link.build();
			info!("You can install me as an app at {link}");
		} else {
			warn!("Couldn't figure out our own client ID! Something might be wrong");
		}
	}
}

#[async_trait]
impl EventHandler for Handler {
	/// Dispatch our commands and try to handle errors from them
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		if let Interaction::Command(command) = interaction {
			let command_name = &command.data.name;
			trace!("Received command: {}", command_name);

			if let Err(why) = Handler::dispatch_command(&ctx, &command).await {
				error!(
					"Ran into an error while dispatching command {}:\n{why:?}",
					command_name
				);

				let embed = CreateEmbed::new()
					.title("An error occurred")
					.description("Sorry about that!")
					.color(Colour::RED);
				let response = CreateInteractionResponseFollowup::new().embed(embed);

				if let Err(why) = command.create_followup(&ctx.http, response).await {
					error!("Ran into an error while trying to recover from an error!\n{why:?}");
				}
			}
		}
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		info!("Connected as {}!", ready.user.name);
		Handler::invite_link(&ctx).await;

		if let Err(why) = self.register_commands(&ctx).await {
			error!("Couldn't register commands!\n{why:?}");
		};
	}
}
