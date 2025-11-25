{
  description = "Rust Dev Shell";

  inputs = {
    unstable.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    unstable,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = unstable.legacyPackages.${system};
        shell = {
          packages = with pkgs; [
            alejandra
            bacon
            cargo
            cargo-edit
            cargo-tarpaulin
            clippy
            rust-analyzer
            rustc
            rustfmt
            license-generator
            djlint
          ];
        };
        mkRustPkg = pkg:
          pkgs.rustPlatform.buildRustPackage {
            name = pkg;
            src = pkgs.nix-gitignore.gitignoreSource [] ./${pkg};
            cargoLock.lockFile = ./${pkg}/Cargo.lock;
          };
      in {
        devShells.default = pkgs.mkShell shell;
        devShells.unfree = pkgs.mkShell (shell
          // {
            packages =
              shell.packages
              ++ [
                pkgs.qutebrowser
                pkgs.antigravity
              ];
          });
        packages = rec {
          site = mkRustPkg "site";
          default = self.packages.${system}.site;
        };
      }
    );
}
