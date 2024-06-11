use crate::{Config, Error, NIXPKGS_BRANCHES, NIXPKGS_REMOTE, NIXPKGS_URL};
use bot_http::{self as http, GithubClientExt};
use git2::Commit;
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

/// Find the branch by it's name and check if it has the commit [`commit`]
///
/// # Errors
///
/// Will return `Err` if the remote branch can't be found by it's name or if we
/// cannot determine if said branch contains the given commit
fn has_commit(
	tracker: &Tracker,
	branch_name: &str,
	commit: &Commit,
) -> Result<bool, git_tracker::Error> {
	trace!("Checking for commit in {branch_name}");
	let full_branch_name = format!("{NIXPKGS_REMOTE}/{branch_name}");
	let branch = tracker.branch_by_name(&full_branch_name)?;
	let has_pr = tracker.ref_contains_commit(&branch.into_reference(), commit)?;

	Ok(has_pr)
}

/// Collect the status of the commit SHA [`commit_sha`] in each of the nixpkgs
/// branches in [`branches`], using the repository at path [`repository_path`]
///
/// # Errors
///
/// Will return `Err` if we can't start tracking a repository at the given path,
/// the commit SHA cannot be found, or if we can't determine if the branch has given
/// commit
fn collect_statuses_in(
	repository_path: &str,
	commit_sha: &str,
	branches: &[&str],
) -> Result<Vec<String>, Error> {
	// start tracking nixpkgs
	let tracker = Tracker::from_path(repository_path)?;

	// find the merge commit
	let commit = tracker.commit_by_sha(commit_sha)?;

	// check to see what branches it's in
	let status_results = branches
		.iter()
		.map(|branch_name| {
			let has_pr = has_commit(&tracker, branch_name, &commit)?;
			Ok(to_status_string(branch_name, has_pr))
		})
		.collect::<Result<Vec<String>, git_tracker::Error>>()?;

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
		let response = CreateInteractionResponseFollowup::new().content(
			"Either this pull request hasn't been merged or it's very old. I can't track it",
		);
		command.create_followup(&ctx, response).await?;

		return Ok(());
	};

	let status_results = collect_statuses_in(&config.nixpkgs_path, &commit_sha, &NIXPKGS_BRANCHES)?;

	let embed = CreateEmbed::new()
		.title(format!("Nixpkgs PR #{} Status", *pr))
		.url(format!("{NIXPKGS_URL}/pull/{pr}"))
		.description(status_results.join("\n"));

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
