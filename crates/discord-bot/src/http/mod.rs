use std::future::Future;

use eyre::Result;
use log::trace;
use serde::de::DeserializeOwned;

mod github;
mod teawie;

pub use github::ClientExt as GithubClientExt;
pub use teawie::ClientExt as TeawieClientExt;

pub type Client = reqwest::Client;
pub type Response = reqwest::Response;

/// Fun trait for functions we use with [Client]
pub trait ClientExt {
	fn default() -> Self;
	fn get_request(&self, url: &str) -> impl Future<Output = Result<Response>> + Send;
	fn get_json<T: DeserializeOwned>(&self, url: &str) -> impl Future<Output = Result<T>> + Send;
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
	/// Will return [`Err`] if the request fails
	async fn get_request(&self, url: &str) -> Result<Response> {
		trace!("Making GET request to {url}");

		let resp = self.get(url).send().await?;
		resp.error_for_status_ref()?;

		Ok(resp)
	}

	/// Perform a GET request to [`url`] and decode the json response
	///
	/// # Errors
	///
	/// Will return [`Err`] if the request fails or cannot be deserialized
	async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
		let resp = self.get_request(url).await?;
		let json = resp.json().await?;
		Ok(json)
	}
}
