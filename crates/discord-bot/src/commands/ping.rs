use eyre::Result;
use serenity::builder::{
	CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::{CommandInteraction, InstallationContext};
use serenity::prelude::Context;

pub async fn respond(ctx: &Context, command: &CommandInteraction) -> Result<()> {
	let message = CreateInteractionResponseMessage::new().content("Pong!");
	let response = CreateInteractionResponse::Message(message);
	command.create_response(&ctx, response).await?;

	Ok(())
}

pub fn register() -> CreateCommand {
	CreateCommand::new("ping")
		.description("Check if the bot is up")
		.add_integration_type(InstallationContext::User)
}
