{
  inputs = {
    nixpkgs.url = sourcehut:~vkleen/nixpkgs/local?host=git.sr.ht.kleen.org;
    utils.url = github:numtide/flake-utils;

    fenix = {
      url = github:nix-community/fenix;
      inputs.nixpkgs.follows=  "nixpkgs";
    };
    naersk = {
      url = github:nix-community/naersk;
      inputs.nixpkgs.follows = "nixpkgs";
    };
    afl = {
      url = git+https://github.com/vkleen/afl.rs?submodules=1;
      flake = false;
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

      cargo-afl = naersk-lib.buildPackage {
        pname = "afl";
        src = inputs.afl;
        RUSTFLAGS = "--cfg docsrs";
        buildInputs = [ pkgs.makeWrapper ];
        postInstall = ''
          cp -r target/release/build/afl-*/out "$out"/
          wrapProgram $out/bin/cargo-afl --set OUT_DIR "$out"/out
        '';
      };

      llvm-config-hack = let
        llvm-paths = pkgs.symlinkJoin {
          name = "llvm-config-hack-path";
          paths = [ pkgs.llvm.dev pkgs.llvm.out pkgs.clang pkgs.lld ];
        };
      in pkgs.writeShellScriptBin "llvm-config" ''
        if [[ "$1" == "--bindir" ]]; then
          echo ${llvm-paths}/bin
        else
          ${pkgs.llvm.dev}/bin/llvm-config "$*"
        fi
      '';
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
            "llvm-tools-preview"
          ])
          cargo-asm cargo-expand cargo-afl cargo-fuzz
          fenix.rust-analyzer
          llvm clang lld
          python3
        ];
        shellHook = ''
          unset CC
          export LLVM_CONFIG=${llvm-config-hack}/bin/llvm-config
        '';
      };

      packages = {
        inherit zorn cargo-afl;
        inherit (fenix) rust-analyzer;
      };
    });
}
