use super::{Error, HttpClientExt};

use serde::Deserialize;

const GITHUB_API: &str = "https://api.github.com";

/// Bad version of `/repos/{owner}/{repo}/{compare}/{ref}...{ref}`
#[derive(Deserialize)]
struct Compare {
	status: String,
	ahead_by: i32,
}

/// Bad version of `/repos/{owner}/{repo}/pulls/{pull_number}`
#[derive(Deserialize)]
struct PullRequest {
	merge_commit_sha: String,
}

pub trait GithubClientExt {
	/// Get the commit that merged [`pr`] in [`repo_owner`]/[`repo_name`]
	async fn merge_commit_for(
		&self,
		repo_owner: &str,
		repo_name: &str,
		pr: u64,
	) -> Result<String, Error>;

	/// Check if [`commit`] is in [`branch`] of [`repo_owner`]/[`repo_name`]
	async fn is_commit_in_branch(
		&self,
		repo_owner: &str,
		repo_name: &str,
		branch_name: &str,
		commit: &str,
	) -> Result<bool, Error>;
}

impl GithubClientExt for super::Client {
	async fn merge_commit_for(
		&self,
		repo_owner: &str,
		repo_name: &str,
		pr: u64,
	) -> Result<String, Error> {
		let url = format!("{GITHUB_API}/repos/{repo_owner}/{repo_name}/pulls/{pr}");
		let resp: PullRequest = self.get_json(&url).await?;
		let merge_commit = resp.merge_commit_sha;

		Ok(merge_commit)
	}

	async fn is_commit_in_branch(
		&self,
		repo_owner: &str,
		repo_name: &str,
		branch: &str,
		commit: &str,
	) -> Result<bool, Error> {
		let url = format!(
			"https://api.github.com/repos/{repo_owner}/{repo_name}/compare/{branch}...{commit}"
		);
		let resp: Compare = self.get_json(&url).await?;
		let in_branch = resp.status != "diverged" && resp.ahead_by >= 0;

		Ok(in_branch)
	}
}
