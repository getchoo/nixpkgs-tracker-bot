use bot_config::Config;
use bot_consts::{NIXPKGS_REMOTE, NIXPKGS_URL};
use bot_error::Error;
use bot_http::{self as http, GithubClientExt};
use git_tracker::Tracker;

use log::trace;
use serenity::all::CreateEmbed;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateInteractionResponseFollowup};
use serenity::model::application::{
	CommandInteraction, CommandOptionType, InstallationContext, ResolvedOption, ResolvedValue,
};
use serenity::prelude::Context;

const REPO_OWNER: &str = "NixOS";
const REPO_NAME: &str = "nixpkgs";

/// Make a nice friendly string displaying if this branch has a PR merged into it
fn to_status_string(branch_name: &str, has_pr: bool) -> String {
	let emoji = if has_pr { "✅" } else { "❌" };
	format!("`{branch_name}` {emoji}")
}

/// Collect the status of the commit SHA [`commit_sha`] in each of the nixpkgs
/// branches in [`branches`], using the repository at path [`repository_path`]
///
/// # Errors
///
/// Will return `Err` if we can't start tracking a repository at the given path,
/// the commit SHA cannot be found, or if we can't determine if the branch has given
/// commit
fn collect_statuses_in<'a>(
	repository_path: &str,
	commit_sha: &str,
	branches: impl IntoIterator<Item = &'a String>,
) -> Result<Vec<String>, Error> {
	// start tracking nixpkgs
	let tracker = Tracker::from_path(repository_path)?;

	// check to see what branches it's in
	let mut status_results = vec![];
	for branch_name in branches {
		trace!("Checking for commit in {branch_name}");
		let full_branch_name = format!("{NIXPKGS_REMOTE}/{branch_name}");
		let has_pr = tracker.branch_contains_sha(&full_branch_name, commit_sha)?;

		if has_pr {
			status_results.push(to_status_string(branch_name, has_pr));
		}
	}

	Ok(status_results)
}

pub async fn respond(
	ctx: &Context,
	http: &http::Client,
	config: &Config,
	command: &CommandInteraction,
) -> Result<(), Error> {
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

	let Ok(pr_id) = u64::try_from(*pr) else {
		let resp =
			CreateInteractionResponseFollowup::new().content("PR numbers aren't negative...");
		command.create_followup(&ctx, resp).await?;

		return Ok(());
	};

	// find out what commit our PR was merged in
	let Some(commit_sha) = http.merge_commit_for(REPO_OWNER, REPO_NAME, pr_id).await? else {
		let response = CreateInteractionResponseFollowup::new()
			.content("It seems this pull request is very old. I can't track it");
		command.create_followup(&ctx, response).await?;

		return Ok(());
	};

	let status_results = collect_statuses_in(
		&config.nixpkgs_path,
		&commit_sha,
		config.nixpkgs_branches.iter(),
	)?;

	let embed_description: String = if status_results.is_empty() {
		"It doesn't look like this PR has been merged yet!".to_string()
	} else {
		status_results.join("\n")
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
