use crate::{config::Config, http::GitHubClientExt};

use std::sync::Arc;
use std::time::Instant;

use eyre::Result;
use log::debug;
use serenity::builder::{
	CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter,
	CreateInteractionResponseFollowup,
};
use serenity::model::{
	application::{
		CommandInteraction, CommandOptionType, InstallationContext, ResolvedOption, ResolvedValue,
	},
	Timestamp,
};
use serenity::prelude::Context;

const REPO_OWNER: &str = "NixOS";
const REPO_NAME: &str = "nixpkgs";

pub async fn respond<T>(
	ctx: &Context,
	http: &Arc<T>,
	config: &Config,
	command: &CommandInteraction,
) -> Result<()>
where
	T: GitHubClientExt,
{
	// this will probably take a while
	command.defer(&ctx).await?;

	let options = command.data.options();
	let Some(ResolvedOption {
		value: ResolvedValue::Integer(pr),
		..
	}) = options.first()
	else {
		let resp = CreateInteractionResponseFollowup::new()
			.content("PR numbers aren't negative or that big...");
		command.create_followup(&ctx, resp).await?;

		return Ok(());
	};

	let Ok(id) = u64::try_from(*pr) else {
		let resp = CreateInteractionResponseFollowup::new()
			.content("PR numbers aren't negative or that big...");
		command.create_followup(&ctx, resp).await?;

		return Ok(());
	};

	// find out what commit our PR was merged in
	let pull_request = http.pull_request(REPO_OWNER, REPO_NAME, id).await?;
	if !pull_request.merged {
		let response = CreateInteractionResponseFollowup::new()
			.content("It looks like that PR isn't merged yet! Try again when it is 😄");
		command.create_followup(&ctx, response).await?;

		return Ok(());
	}

	// seems older PRs may not have this
	let Some(commit_sha) = pull_request.merge_commit_sha else {
		let response = CreateInteractionResponseFollowup::new()
			.content("It seems this pull request is very old. I can't track it");
		command.create_followup(&ctx, response).await?;

		return Ok(());
	};

	let repository = config.repository();
	let timer = Instant::now();
	let branch_results = repository.branches_contain_sha(config.nixpkgs_branches(), &commit_sha)?;
	let branch_check_time = timer.elapsed();
	let fields: Vec<_> = branch_results
		.iter()
		.map(|(name, has_commit)| {
			let emoji = if *has_commit { "✅" } else { "❌" };
			(*name, emoji, true)
		})
		.collect();

	// if we didn't find any, bail
	if fields.is_empty() {
		let response = CreateInteractionResponseFollowup::new()
			.content("This PR has been merged...but I can't seem to find it anywhere. I might not be tracking it's base branch");
		command.create_followup(&ctx, response).await?;

		return Ok(());
	}

	let mut embed = CreateEmbed::new()
		.title(format!("Nixpkgs PR #{} Status", pull_request.number))
		.url(&pull_request.html_url)
		.description(&pull_request.title)
		.fields(fields)
		.footer(CreateEmbedFooter::new(format!(
			"Completed in {}ms",
			branch_check_time.as_millis()
		)));

	if let Some(merged_at) = pull_request.merged_at {
		if let Ok(timestamp) = Timestamp::parse(&merged_at) {
			embed = embed.timestamp(timestamp);
		} else {
			debug!("Couldn't parse timestamp from GitHub! Ignoring.");
		}
	} else {
		debug!("Couldn't find `merged_at` information for a supposedly merged PR! Ignoring.");
	}

	let resp = CreateInteractionResponseFollowup::new().embed(embed);
	command.create_followup(&ctx, resp).await?;

	Ok(())
}

pub fn register() -> CreateCommand {
	CreateCommand::new("track")
		.description("Track a nixpkgs PR")
		.add_integration_type(InstallationContext::User)
		.add_option(
			CreateCommandOption::new(CommandOptionType::Integer, "pull_request", "PR to track")
				.required(true),
		)
}
