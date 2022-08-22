{
  inputs = {
    nixpkgs.url = sourcehut:~vkleen/nixpkgs/local?host=git.sr.ht.kleen.org;
    utils.url = github:numtide/flake-utils;

    fenix = {
      url = github:nix-community/fenix;
      inputs.nixpkgs.follows=  "nixpkgs";
    };
    naersk = {
      url = github:nmattia/naersk;
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, ... }@inputs:
    utils.lib.eachDefaultSystem (system: let
      inherit (nixpkgs) lib;
      pkgs = import "${nixpkgs}/pkgs/top-level" {
        localSystem = { inherit system; };
      };

      fenix-profile = "latest";

      fenix = inputs.fenix.packages.${system};
      fenix-toolchain = fenix.${fenix-profile};
      naersk-lib = inputs.naersk.lib.${system}.override { inherit (fenix-toolchain) cargo rustc; };

      workspacePackage = crate: args: naersk-lib.buildPackage ({
        pname = crate;
        src = ./.;
        cargoBuildOptions = x: x ++ [ "-p" crate ];
        cargoTestOptions = x: x ++ [ "-p" crate ];
      } // args);

      zorn = workspacePackage "zorn" { };
    in {
      defaultPackage = zorn;

      apps = {
        zorn = utils.lib.mkApp { drv = zorn; };
      };

      devShell = with pkgs; mkShell {
        inputsFrom = lib.attrValues self.packages.${system};
        packages = [
          (fenix-toolchain.withComponents [
            "cargo" "clippy" "rust-src" "rustc" "rustfmt"
          ])
          cargo-asm cargo-expand
          fenix.rust-analyzer
        ];
      };

      packages = {
        inherit zorn;
        inherit (fenix) rust-analyzer;
      };
    });
}
