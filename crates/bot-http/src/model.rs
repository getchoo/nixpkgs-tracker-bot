use serde::Deserialize;

/// Bad version of `/repos/{owner}/{repo}/pulls/{pull_number}`
#[derive(Deserialize)]
pub struct PullRequest {
	pub merge_commit_sha: Option<String>,
}
