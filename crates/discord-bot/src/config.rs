use git_tracker::TrackedRepository;

use std::{env, path::PathBuf, sync::Arc};

const DEFAULT_NIXPKGS_URL: &str = "https://github.com/NixOS/nixpkgs";

const DEFAULT_NIXPKGS_REMOTE: &str = "origin";

/// The Discord client's configuration
#[derive(Clone, Debug)]
pub struct Config {
	/// Comma separated list of nixpkgs branch to track commits for
	nixpkgs_branches: Vec<String>,
	/// Repository tracker
	repository: Arc<TrackedRepository>,
}

impl Config {
	/// Create a new instance of [`Config`] based on variables from the environment
	///
	/// # Errors
	///
	/// Will return [`Err`] if a variable is not found
	pub fn from_env() -> Result<Self, env::VarError> {
		let nixpkgs_path = env::var("BOT_NIXPKGS_PATH")?;

		let nixpkgs_branches = env::var("BOT_NIXPKGS_BRANCHES")?
			.split(',')
			.map(ToString::to_string)
			.collect();

		let nixpkgs_remote =
			env::var("BOT_NIXPKGS_REMOTE").unwrap_or(DEFAULT_NIXPKGS_REMOTE.to_string());
		let nixpkgs_url = env::var("BOT_NIXPKGS_URL").unwrap_or(DEFAULT_NIXPKGS_URL.to_string());

		let repository = TrackedRepository::new(
			PathBuf::from(nixpkgs_path.clone()),
			nixpkgs_url,
			nixpkgs_remote,
		);

		Ok(Self {
			nixpkgs_branches,
			repository: Arc::new(repository),
		})
	}

	pub fn repository(&self) -> &TrackedRepository {
		&self.repository
	}

	pub fn nixpkgs_branches(&self) -> &Vec<String> {
		&self.nixpkgs_branches
	}
}
