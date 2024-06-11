use std::path::Path;

use git2::{Branch, BranchType, Commit, Oid, Reference, Repository};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("libgit2 error")]
	Git(#[from] git2::Error),
	#[error("Repository path not found at `{0}`")]
	RepositoryPathNotFound(String),
}

/// Helper struct for tracking Git objects
pub struct Tracker {
	repository: Repository,
}

impl Tracker {
	/// Create a new [`Tracker`] using the repository at [`path`]
	///
	/// # Errors
	///
	/// Will return `Err` if the repository can not be opened
	pub fn from_path(path: &str) -> Result<Self, Error> {
		let repository_path = Path::new(path);
		if repository_path.exists() {
			let repository = Repository::open(repository_path)?;
			Ok(Self { repository })
		} else {
			Err(Error::RepositoryPathNotFound(path.to_string()))
		}
	}

	/// Finds a branch of name [`name`]
	///
	/// # Errors
	///
	/// Will return `Err` if the branch cannot be found locally
	pub fn branch_by_name(&self, name: &str) -> Result<Branch, Error> {
		Ok(self.repository.find_branch(name, BranchType::Remote)?)
	}

	/// Finds a commit with a SHA match [`sha`]
	///
	/// # Errors
	///
	/// Will return `Err` if [`sha`] cannot be converted an [`Oid`] or
	/// a commit matching it cannot be found
	pub fn commit_by_sha(&self, sha: &str) -> Result<Commit, Error> {
		let oid = Oid::from_str(sha)?;
		let commit = self.repository.find_commit(oid)?;

		Ok(commit)
	}

	/// Check if [`Reference`] [`ref`] contains [`Commit`] [`commit`]
	///
	/// # Errors
	///
	/// Will return `Err` if the reference cannot be resolved to a commit
	pub fn ref_contains_commit(
		&self,
		reference: &Reference,
		commit: &Commit,
	) -> Result<bool, Error> {
		let head = reference.peel_to_commit()?;
		let has_commit = self
			.repository
			.graph_descendant_of(head.id(), commit.id())?;

		Ok(has_commit)
	}
}
