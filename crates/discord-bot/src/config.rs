use std::env;

/// The Discord client's configuration
#[derive(Clone, Debug)]
pub struct Config {
	/// Path to clone a new or use an existing nixpkgs repository
	pub nixpkgs_path: String,
	// A comma separated list of nixpkgs branch to track commits for
	pub nixpkgs_branches: Vec<String>,
}

impl Config {
	/// Take in a comma separated list and split it into a [`Vec<String>`]
	fn split_string_list(branches: &str) -> Vec<String> {
		branches
			.split(',')
			.map(|branch| branch.trim().to_string())
			.collect()
	}

	/// Create a new instance of [`Config`] based on variables from the environment
	///
	/// # Errors
	///
	/// Will return [`Err`] if a variable is not found
	pub fn from_env() -> Result<Self, env::VarError> {
		let nixpkgs_path = env::var("BOT_NIXPKGS_PATH")?;
		let nixpkgs_branches_raw = env::var("BOT_NIXPKGS_BRANCHES")?;
		let nixpkgs_branches = Self::split_string_list(&nixpkgs_branches_raw);

		Ok(Self {
			nixpkgs_path,
			nixpkgs_branches,
		})
	}
}
