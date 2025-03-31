{
  description = "A Discord app for tracking nixpkgs PRs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    # Inputs below this are optional
    # `inputs.treefmt-nix.follows = ""`

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
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
      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {
          clippy-sarif = pkgs.stdenv.mkDerivation {
            name = "check-clippy-sarif";
            inherit (self.packages.${system}.nixpkgs-tracker-bot) src cargoDeps;

            nativeBuildInputs = with pkgs; [
              cargo
              clippy
              clippy-sarif
              pkg-config
              rustPlatform.cargoSetupHook
              rustc
              sarif-fmt
            ];

            buildInputs = [ pkgs.openssl ];

            buildPhase = ''
              cargo clippy \
                --all-features \
                --all-targets \
                --tests \
                --message-format=json \
              | clippy-sarif | tee $out | sarif-fmt
            '';
          };

          treefmt = treefmtFor.${system}.config.build.check self;
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.clippy
              pkgs.rustfmt
              pkgs.rust-analyzer

              self.formatter.${system}
            ];

            inputsFrom = [ self.packages.${system}.nixpkgs-tracker-bot ];
            env = {
              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            };
          };
        }
      );

      formatter = forAllSystems (system: treefmtFor.${system}.config.build.wrapper);

      nixosModules.default = import ./nix/module.nix self;

      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          packages' = self.packages.${system};

          staticWith = pkgs.callPackage ./nix/static.nix { inherit self; };
          containerize = pkgs.callPackage ./nix/containerize.nix { };
        in
        {
          nixpkgs-tracker-bot = pkgs.callPackage ./nix/package.nix { inherit self; };

          default = packages'.nixpkgs-tracker-bot;

          static-x86_64 = staticWith { arch = "x86_64"; };
          static-arm64 = staticWith { arch = "aarch64"; };

          container-amd64 = containerize packages'.static-x86_64;
          container-arm64 = containerize packages'.static-arm64;
        }
      );
    };
}
