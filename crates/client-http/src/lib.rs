mod github;
mod model;
mod teawie;

pub use github::Ext as GitHubClientExt;
pub use model::*;
pub use teawie::Ext as TeawieClientExt;

pub type Client = reqwest::Client;
pub type Error = reqwest::Error;

/// Fun trait for functions we use with [Client]
pub trait Ext {
	fn default() -> Self;
}

impl Ext for Client {
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
}
