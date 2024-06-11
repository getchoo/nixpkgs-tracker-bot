use std::future::Future;

use log::trace;
use serde::de::DeserializeOwned;

mod github;
mod model;

pub use github::ClientExt as GithubClientExt;

pub type Client = reqwest::Client;
pub type Response = reqwest::Response;
pub type Error = reqwest::Error;

/// Fun trait for functions we use with [Client]
pub trait ClientExt {
	fn default() -> Self;
	fn get_request(&self, url: &str) -> impl Future<Output = Result<Response, Error>> + Send;
	fn get_json<T: DeserializeOwned>(
		&self,
		url: &str,
	) -> impl Future<Output = Result<T, Error>> + Send;
}

impl ClientExt for Client {
	/// Create the default [`Client`]
	fn default() -> Self {
		reqwest::Client::builder()
			.user_agent(format!(
				"nixpkgs-tracker-bot/{}",
				option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "development")
			))
			.build()
			.unwrap()
	}

	/// Perform a GET request to [`url`]
	///
	/// # Errors
	///
	/// Will return `Err` if the request fails
	async fn get_request(&self, url: &str) -> Result<Response, Error> {
		trace!("Making GET request to {url}");

		let resp = self.get(url).send().await?;
		resp.error_for_status_ref()?;

		Ok(resp)
	}

	/// Perform a GET request to [`url`] and decode the json response
	///
	/// # Errors
	///
	/// Will return `Err` if the request fails or cannot be deserialized
	async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
		let resp = self.get_request(url).await?;
		let json = resp.json().await?;
		Ok(json)
	}
}
