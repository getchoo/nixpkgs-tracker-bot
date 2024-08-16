use std::sync::Arc;

use crate::{config::Config, consts::NIXPKGS_URL, http::GitHubClientExt};

use eyre::Result;
use serenity::all::CreateEmbed;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateInteractionResponseFollowup};
use serenity::model::application::{
	CommandInteraction, CommandOptionType, InstallationContext, ResolvedOption, ResolvedValue,
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
			.content("Please provide a valid pull request!");
		command.create_followup(&ctx, resp).await?;

		return Ok(());
	};

	let Ok(id) = u64::try_from(*pr) else {
		let resp =
			CreateInteractionResponseFollowup::new().content("PR numbers aren't negative...");
		command.create_followup(&ctx, resp).await?;

		return Ok(());
	};

	// find out what commit our PR was merged in
	let pull_request = http.pull_request(REPO_OWNER, REPO_NAME, id).await?;
	let Some(commit_sha) = pull_request.merge_commit_sha else {
		let response = CreateInteractionResponseFollowup::new()
			.content("It seems this pull request is very old. I can't track it");
		command.create_followup(&ctx, response).await?;

		return Ok(());
	};

	let status_results = git_tracker::collect_statuses_in(
		&config.nixpkgs_path,
		&commit_sha,
		&config.nixpkgs_branches,
	)?;

	// if we don't find the commit in any branches from above, we can pretty safely assume
	// it's an unmerged PR
	let embed_description: String = if status_results.is_empty() {
		"It doesn't look like this PR has been merged yet! (or maybe I just haven't updated)"
			.to_string()
	} else {
		status_results
			.iter()
			.filter_map(|(branch_name, has_pr)| has_pr.then(|| format!("`{branch_name}` ✅")))
			.collect::<Vec<String>>()
			.join("\n")
	};

	let embed = CreateEmbed::new()
		.title(format!("Nixpkgs PR #{} Status", *pr))
		.url(format!("{NIXPKGS_URL}/pull/{pr}"))
		.description(embed_description);

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
