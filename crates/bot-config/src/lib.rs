use std::env;

/// The Discord client's configuration
#[derive(Clone, Debug)]
pub struct Config {
	pub nixpkgs_path: String,
}

impl Config {
	/// Create a new instance of [`Config`] based on variables from the environment
	///
	/// # Errors
	///
	/// Will return [`Err`] if a variable is not found
	pub fn from_env() -> Result<Self, env::VarError> {
		let nixpkgs_path = env::var("BOT_NIXPKGS_PATH")?;

		Ok(Self { nixpkgs_path })
	}
}
