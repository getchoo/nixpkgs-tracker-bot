use serenity::builder::CreateCommand;

pub mod ping;
pub mod track;

macro_rules! cmd {
	($module: ident) => {
		$module::register()
	};
}

/// Return a list of all our [`CreateCommand`]s
#[must_use]
pub fn to_vec() -> Vec<CreateCommand> {
	vec![cmd!(ping), cmd!(track)]
}
