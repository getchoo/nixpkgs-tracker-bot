{
  description = "A Discord app for tracking nixpkgs PRs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-analyzer-src.follows = "";
      };
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      treefmt-nix,
    }:
    let
      inherit (nixpkgs) lib;
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = lib.genAttrs systems;
      nixpkgsFor = forAllSystems (system: nixpkgs.legacyPackages.${system});
      treefmtFor = forAllSystems (system: treefmt-nix.lib.evalModule nixpkgsFor.${system} ./treefmt.nix);
    in
    {
      checks = forAllSystems (system: {
        treefmt = treefmtFor.${system}.config.build.check self;
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          inputsFrom = [ self.packages.${system}.nixpkgs-tracker-bot ];
        in
        {
          default = pkgs.mkShell {
            inherit inputsFrom;
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

            packages = [
              pkgs.clippy
              pkgs.rustfmt
              pkgs.rust-analyzer

              self.formatter.${system}
            ];
          };

          ci = pkgs.mkShell {
            inherit inputsFrom;
            packages = [
              pkgs.clippy
              pkgs.rustfmt
            ];
          };
        }
      );

      formatter = forAllSystems (system: treefmtFor.${system}.config.build.wrapper);

      nixosModules.default = import ./nix/module.nix self;

      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          packages = self.packages.${system};

          mkStaticWith = pkgs.callPackage ./nix/static.nix {
            inherit (packages) nixpkgs-tracker-bot;
            fenix = fenix.packages.${system};
          };

          containerWith =
            nixpkgs-tracker-bot:
            let
              arch = nixpkgs-tracker-bot.stdenv.hostPlatform.ubootArch;
            in
            pkgs.dockerTools.buildLayeredImage {
              name = "nixpkgs-tracker-bot";
              tag = "latest-${arch}";
              config.Cmd = [ (lib.getExe nixpkgs-tracker-bot) ];
              architecture = arch;
            };
        in
        {
          nixpkgs-tracker-bot = pkgs.callPackage ./nix/package.nix {
            version = self.shortRev or self.dirtyShortRev or "unknown";
          };

          default = packages.nixpkgs-tracker-bot;

          static-x86_64 = mkStaticWith { arch = "x86_64"; };
          static-arm64 = mkStaticWith { arch = "aarch64"; };

          container-x86_64 = containerWith packages.static-x86_64;
          container-arm64 = containerWith packages.static-arm64;
        }
      );
    };
}
