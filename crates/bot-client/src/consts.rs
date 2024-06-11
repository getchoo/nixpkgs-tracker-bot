/// All of our tracked branches in nixpkgs
pub const NIXPKGS_BRANCHES: [&str; 7] = [
	"master",
	"staging",
	"nixpkgs-unstable",
	"nixos-unstable",
	"nixos-unstable-small",
	"nixos-24.05-small",
	"nixos-24.05",
];

/// URL to the nixpkgs repository
pub const NIXPKGS_URL: &str = "https://github.com/NixOS/nixpkgs";

/// The Git remote for upstream nixpkgs in our local copy
pub const NIXPKGS_REMOTE: &str = "origin";
