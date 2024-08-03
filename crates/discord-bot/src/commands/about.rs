use std::sync::Arc;

use crate::http::TeawieClientExt;

use eyre::Result;
use log::warn;
use serenity::builder::{
	CreateCommand, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};
use serenity::model::application::{CommandInteraction, InstallationContext};
use serenity::prelude::Context;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub async fn respond<T>(ctx: &Context, http: &Arc<T>, command: &CommandInteraction) -> Result<()>
where
	T: TeawieClientExt,
{
	let mut embed = CreateEmbed::new()
		.title("About nixpkgs-tracker-bot")
		.description("I help track what branches PRs to nixpkgs have reached. If you've used [Nixpkgs Pull Request Tracker](https://nixpk.gs/pr-tracker.html), you probably know what this is about.")
		.fields([
			("Version", VERSION, true),
			("Source code", &format!("[getchoo/nixpkgs-tracker-bot]({REPOSITORY})"), true),
			("Issues/Feature Requests", &format!("[getchoo/nixpkgs-tracker-bot/issues]({REPOSITORY}/issues)"), true)
	]);

	let random_teawie = http.random_teawie().await?;

	if let Some(teawie_url) = random_teawie.url {
		let footer = CreateEmbedFooter::new("Images courtesy of @sympathytea");
		embed = embed.image(teawie_url).footer(footer);
	} else if let Some(error) = random_teawie.error {
		warn!("Error from TeawieAPI: {error:#?}");
	};

	let message = CreateInteractionResponseMessage::new().embed(embed);
	let response = CreateInteractionResponse::Message(message);
	command.create_response(&ctx, response).await?;

	Ok(())
}

pub fn register() -> CreateCommand {
	CreateCommand::new("about")
		.description("Learn more about me")
		.add_integration_type(InstallationContext::User)
}
