use super::{ClientExt as _, Error};
use crate::model::PullRequest;

use std::future::Future;

const GITHUB_API: &str = "https://api.github.com";

pub trait ClientExt {
	/// Get the commit that merged [`pr`] in [`repo_owner`]/[`repo_name`]
	///
	/// # Errors
	///
	/// Will return [`Err`] if the merge commit cannot be found
	fn merge_commit_for(
		&self,
		repo_owner: &str,
		repo_name: &str,
		pr: u64,
	) -> impl Future<Output = Result<Option<String>, Error>> + Send;
}

impl ClientExt for super::Client {
	async fn merge_commit_for(
		&self,
		repo_owner: &str,
		repo_name: &str,
		pr: u64,
	) -> Result<Option<String>, Error> {
		let url = format!("{GITHUB_API}/repos/{repo_owner}/{repo_name}/pulls/{pr}");
		let resp: PullRequest = self.get_json(&url).await?;
		let merge_commit = resp.merge_commit_sha;

		Ok(merge_commit)
	}
}
