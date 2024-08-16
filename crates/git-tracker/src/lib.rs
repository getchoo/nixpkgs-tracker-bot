//! A library that helps you track commits and branches in a Git repository
use std::collections::HashMap;

use log::trace;

mod managed_repository;
mod tracker;
pub use managed_repository::ManagedRepository;
pub use tracker::Tracker;

/// Collect the status of the commit SHA [`commit_sha`] in each of the nixpkgs
/// branches in [`branches`], using the repository at path [`repository_path`]
///
/// NOTE: `branches` should contain the full ref (i.e., `origin/main`)
///
/// # Errors
///
/// Will return [`Err`] if we can't start tracking a repository at the given path,
/// or if we can't determine if the branch has given commit
pub fn collect_statuses_in(
	repository_path: &str,
	commit_sha: &str,
	branches: &Vec<String>,
) -> Result<HashMap<String, bool>, tracker::Error> {
	// start tracking nixpkgs
	let tracker = Tracker::from_path(repository_path)?;

	// check to see what branches it's in
	let mut status_results = HashMap::new();
	for branch_name in branches {
		trace!("Checking for commit in {branch_name}");
		let has_pr = tracker.branch_contains_sha(branch_name, commit_sha)?;
		status_results.insert(branch_name.to_string(), has_pr);
	}

	Ok(status_results)
}
