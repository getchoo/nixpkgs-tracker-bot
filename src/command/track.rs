use crate::http::{Client, GithubClientExt};

use eyre::Result;
use futures::future::try_join_all;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateInteractionResponseFollowup};
use serenity::model::application::{
	CommandInteraction, CommandOptionType, InstallationContext, ResolvedOption, ResolvedValue,
};
use serenity::prelude::Context;
use tracing::{instrument, trace};

/// All of our tracked branches in nixpkgs
const BRANCHES: [&str; 8] = [
	"master",
	"staging",
	"nixos-unstable",
	"nixos-unstable-small",
	"nixos-24.05-small",
	"release-24.05",
	"nixos-23.11-small",
	"release-23.11",
];

#[derive(Clone, Debug, Default)]
struct BranchStatus {
	repo_owner: String,
	repo_name: String,
	name: String,
}

impl BranchStatus {
	fn new(repo_owner: String, repo_name: String, name: String) -> Self {
		Self {
			repo_owner,
			repo_name,
			name,
		}
	}

	/// Make a nice friendly string displaying if this branch has a PR merged into it
	fn to_status_string(&self, has_pr: bool) -> String {
		let emoji = if has_pr { "✅" } else { "❌" };
		format!("`{}` {emoji}", &self.name)
	}

	/// Check if this branch has the specified pull request merged into it
	#[instrument(skip(http))]
	async fn has_pr(&self, http: &Client, pr: u64) -> Result<bool> {
		let commit = http
			.merge_commit_for(
				&self.repo_owner,
				&self.repo_name,
				u64::try_from(pr).unwrap(),
			)
			.await?;

		let has_pr = http
			.is_commit_in_branch(&self.repo_owner, &self.repo_name, &self.name, &commit)
			.await?;

		Ok(has_pr)
	}
}

/// async wrapper for [BranchStatus::to_status_string()]
#[instrument(skip(http))]
async fn collect_status(
	http: &Client,
	repo_owner: String,
	repo_name: String,
	branch: String,
	pr: u64,
) -> Result<String> {
	let status = BranchStatus::new(repo_owner, repo_name, branch);
	let has_pr = status.has_pr(http, pr).await?;
	let res = status.to_status_string(has_pr);

	Ok(res)
}

#[instrument(skip_all)]
pub async fn respond(ctx: &Context, http: &Client, command: &CommandInteraction) -> Result<()> {
	trace!("Responding to track command");

	// this will probably take a while
	command.defer(&ctx).await?;

	// TODO: make these configurable for nixpkgs forks...or other github repos ig
	const REPO_OWNER: &str = "NixOS";
	const REPO_NAME: &str = "nixpkgs";

	let options = command.data.options();

	let response = if let Some(ResolvedOption {
		value: ResolvedValue::Integer(pr),
		..
	}) = options.first()
	{
		if *pr < 0 {
			CreateInteractionResponseFollowup::new().content("PR numbers aren't negative...")
		} else {
			// TODO: this is gross
			let statuses = try_join_all(BRANCHES.iter().map(|&branch| {
				collect_status(
					http,
					REPO_OWNER.to_string(),
					REPO_NAME.to_string(),
					branch.to_string(),
					u64::try_from(*pr).unwrap(),
				)
			}))
			.await?;

			CreateInteractionResponseFollowup::new().content(statuses.join("\n"))
		}
	} else {
		CreateInteractionResponseFollowup::new().content("Please provide a valid commit!")
	};

	command.create_followup(&ctx, response).await?;

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
