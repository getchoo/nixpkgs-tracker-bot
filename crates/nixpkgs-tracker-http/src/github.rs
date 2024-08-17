use super::{Error, PullRequest};

use std::future::Future;

use log::trace;

const GITHUB_API: &str = "https://api.github.com";

pub trait Ext {
	/// GET `/repos/{repo_owner}/{repo_name}/pulls/{id}`
	///
	/// # Errors
	///
	/// Will return [`Err`] if the merge commit cannot be found
	fn pull_request(
		&self,
		repo_owner: &str,
		repo_name: &str,
		id: u64,
	) -> impl Future<Output = Result<PullRequest, Error>> + Send;
}

impl Ext for super::Client {
	async fn pull_request(
		&self,
		repo_owner: &str,
		repo_name: &str,
		id: u64,
	) -> Result<PullRequest, Error> {
		let url = format!("{GITHUB_API}/repos/{repo_owner}/{repo_name}/pulls/{id}");

		let request = self.get(&url).build()?;
		trace!("Making GET request to `{}`", request.url());
		let response = self.execute(request).await?;
		response.error_for_status_ref()?;
		let pull_request: PullRequest = response.json().await?;

		Ok(pull_request)
	}
}
