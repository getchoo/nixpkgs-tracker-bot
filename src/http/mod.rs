use serde::de::DeserializeOwned;
use tracing::trace;

mod github;

pub use github::*;

pub type Client = reqwest::Client;
pub type Response = reqwest::Response;
pub type Error = reqwest::Error;

/// Fun trait for functions we use with [Client]
pub trait HttpClientExt {
	fn default() -> Self;
	async fn get_request(&self, url: &str) -> Result<Response, Error>;
	async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, Error>;
}

impl HttpClientExt for Client {
	fn default() -> Self {
		reqwest::Client::builder()
			.user_agent(format!(
				"nixpkgs-tracker-bot/{}",
				option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "development")
			))
			.build()
			.unwrap()
	}

	async fn get_request(&self, url: &str) -> Result<Response, Error> {
		trace!("Making GET request to {url}");

		let resp = self.get(url).send().await?;
		resp.error_for_status_ref()?;

		Ok(resp)
	}

	async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
		let resp = self.get_request(url).await?;
		let json = resp.json().await?;
		Ok(json)
	}
}
