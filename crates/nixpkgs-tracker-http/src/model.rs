use serde::Deserialize;

/// Bad version of `/repos/{owner}/{repo}/pulls/{pull_number}` for Github's api
#[derive(Clone, Debug, Deserialize)]
pub struct PullRequest {
	pub html_url: String,
	pub number: u64,
	pub title: String,
	pub merged: bool,
	pub merged_at: Option<String>,
	pub merge_commit_sha: Option<String>,
}

/// `/random_teawie` for the teawieAPI
#[derive(Clone, Debug, Deserialize)]
pub struct RandomTeawie {
	pub url: Option<String>,
	pub error: Option<String>,
}
