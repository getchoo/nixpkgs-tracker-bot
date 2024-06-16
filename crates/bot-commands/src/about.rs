use bot_error::Error;
use bot_http::TeawieClientExt;

use serenity::builder::{
	CreateCommand, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};
use serenity::model::application::{CommandInteraction, InstallationContext};
use serenity::prelude::Context;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub async fn respond(
	ctx: &Context,
	http: &bot_http::Client,
	command: &CommandInteraction,
) -> Result<(), Error> {
	let mut embed = CreateEmbed::new()
		.title("About nixpkgs-tracker-bot")
		.description("I help track what branches PRs to nixpkgs have reached. If you've used [Nixpkgs Pull Request Tracker](https://nixpk.gs/pr-tracker.html), you probably know what this is about.")
		.fields([
			("Version", VERSION, true),
			("Source code", &format!("[getchoo/nixpkgs-tracker-bot]({REPOSITORY})"), true),
			("Issues/Feature Requests", &format!("[getchoo/nixpkgs-tracker-bot/issues]({REPOSITORY}/issues)"), true)
	]);

	if let Some(teawie_url) = http.random_teawie().await? {
		let footer = CreateEmbedFooter::new("Images courtesy of @sympathytea");
		embed = embed.image(teawie_url).footer(footer);
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
