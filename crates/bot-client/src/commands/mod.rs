use crate::{Error, SharedConfig, SharedHttp};

use serenity::builder::{
	CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;

mod ping;
mod track;

macro_rules! cmd {
	($module: ident) => {
		$module::register()
	};
}

/// Return a list of all our [`CreateCommand`]s
pub fn to_vec() -> Vec<CreateCommand> {
	vec![cmd!(ping), cmd!(track)]
}

/// Dispatch our commands from a [`CommandInteraction`]
pub async fn dispatch(ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
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
		"ping" => ping::respond(ctx, command).await?,
		"track" => track::respond(ctx, &http, &config, command).await?,
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
