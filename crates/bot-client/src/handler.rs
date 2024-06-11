use crate::commands;

use std::error::Error;

use log::{debug, error, info, trace};
use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateInteractionResponseFollowup};
use serenity::model::{
	application::{Command, Interaction},
	colour::Colour,
	gateway::Ready,
};
use serenity::prelude::{Context, EventHandler};

#[derive(Clone, Copy, Debug)]
pub struct Handler;

impl Handler {
	async fn register_commands(&self, ctx: &Context) -> Result<(), Box<dyn Error>> {
		let commands = commands::to_vec();
		let commands_len = commands.len();
		for command in commands {
			Command::create_global_command(&ctx.http, command).await?;
		}

		debug!("Registered {} commands", commands_len);
		Ok(())
	}
}

#[async_trait]
impl EventHandler for Handler {
	/// Dispatch our commands and try to handle errors from them
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		if let Interaction::Command(command) = interaction {
			let command_name = &command.data.name;
			trace!("Received command: {}", command_name);

			if let Err(why) = commands::dispatch(&ctx, &command).await {
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

		if let Err(why) = self.register_commands(&ctx).await {
			error!("Couldn't register commands!\n{why:?}");
		};
	}
}
