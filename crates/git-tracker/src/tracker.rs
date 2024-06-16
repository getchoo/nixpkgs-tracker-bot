use std::path::Path;

use git2::{Branch, BranchType, Commit, ErrorCode, Oid, Reference, Repository};

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
	/// Will return [`Err`] if the repository can not be opened
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
	/// Will return [`Err`] if the branch cannot be found locally
	pub fn branch_by_name(&self, name: &str) -> Result<Branch, Error> {
		Ok(self.repository.find_branch(name, BranchType::Remote)?)
	}

	/// Finds a commit with a SHA match [`sha`]
	///
	/// # Errors
	///
	/// Will return [`Err`] if [`sha`] cannot be converted an [`Oid`] or
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
	/// Will return [`Err`] if the reference cannot be resolved to a commit or the descendants
	/// of the reference cannot be resolved
	pub fn ref_contains_commit(
		&self,
		reference: &Reference,
		commit: &Commit,
	) -> Result<bool, Error> {
		let head = reference.peel_to_commit()?;

		// NOTE: we have to check this as `Repository::graph_descendant_of()` (like the name says)
		// only finds *descendants* of it's parent commit, and will not tell us if the parent commit
		// *is* the child commit. i have no idea why i didn't think of this, but that's why this
		// comment is here now
		let is_head = head.id() == commit.id();

		let has_commit = self
			.repository
			.graph_descendant_of(head.id(), commit.id())?;

		Ok(has_commit || is_head)
	}

	/// Check if a [`Branch`] named [`branch_name`] has a commit with the SHA [`commit_sha`]
	///
	/// # Errors
	///
	/// Will return [`Err`] if the commit SHA cannot be resolved to an object id, the branch name cannot
	/// be resolved to a branch, or the descendants of the resolved branch cannot be resolved
	pub fn branch_contains_sha(&self, branch_name: &str, commit_sha: &str) -> Result<bool, Error> {
		let commit = match self.commit_by_sha(commit_sha) {
			Ok(commit) => commit,
			Err(why) => {
				// NOTE: we assume commits not found are just not in the branch *yet*, not an error
				// this is because github decides to report merge commit shas for unmerged PRs...yeah
				if let Error::Git(git_error) = &why {
					if git_error.code() == ErrorCode::NotFound {
						return Ok(false);
					}
				}

				return Err(why);
			}
		};

		let branch = self.branch_by_name(branch_name)?;
		let has_pr = self.ref_contains_commit(&branch.into_reference(), &commit)?;

		Ok(has_pr)
	}
}
