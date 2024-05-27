use eyre::{OptionExt, Result};
use serenity::builder::{
	CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;
use tracing::instrument;

use crate::client::SharedClient;

mod ping;
mod track;

macro_rules! cmd {
	($module: ident) => {
		$module::register()
	};
}

/// Return a list of all our [CreateCommand]s
pub fn to_vec() -> Vec<CreateCommand> {
	vec![cmd!(ping), cmd!(track)]
}

/// Dispatch our commands from a [CommandInteraction]
#[instrument(skip(ctx))]
pub async fn dispatch(ctx: &Context, command: &CommandInteraction) -> Result<()> {
	let command_name = command.data.name.as_str();

	// grab our http client from the aether
	let http = {
		let read = ctx.data.read().await;
		read.get::<SharedClient>()
			.ok_or_eyre("Couldn't get shared HTTP client! WHY??????")?
			.clone()
	};

	match command_name {
		"ping" => ping::respond(ctx, command).await?,
		"track" => track::respond(ctx, &http, command).await?,
		_ => {
			let message = CreateInteractionResponseMessage::new().content(format!(
				"It doesn't look like you can use `{command_name}`. Sorry :("
			));
			let response = CreateInteractionResponse::Message(message);
			command.create_response(&ctx, response).await?
		}
	};

	Ok(())
}
