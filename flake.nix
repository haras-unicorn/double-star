{
  description = "double-star";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    cargo2nix.inputs.nixpkgs.follows = "nixpkgs";
    cargo2nix.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = { nixpkgs, flake-utils, cargo2nix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = { allowUnfree = true; };
          overlays = [ cargo2nix.overlays.default ];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.75.0";
          packageFun = import ./Cargo.nix;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          shellHook = ''
            db="$(git rev-parse --show-toplevel)/scripts/db.nu"

            docker compose up -d
        
            DOUBLE_STAR_DB_HOST="$($db host)"
            export DOUBLE_STAR_DB_HOST
            echo "DOUBLE_STAR_DB_HOST is set to $DOUBLE_STAR_DB_HOST"

            DOUBLE_STAR_DB_PORT="$($db port)"
            export DOUBLE_STAR_DB_PORT
            echo "DOUBLE_STAR_DB_PORT is set to $DOUBLE_STAR_DB_PORT"

            $db isready
          '';

          packages = with pkgs; [
            # versioning
            git

            # scripts
            just
            nushell

            # spelling
            nodePackages.cspell

            # tools
            pkg-config
            openssl
            jq

            # markdown
            marksman
            markdownlint-cli
            nodePackages.markdown-link-check

            # misc
            nodePackages.prettier
            nodePackages.yaml-language-server
            nodePackages.vscode-langservers-extracted
            taplo

            # nix
            nil
            nixpkgs-fmt
            cargo2nix.packages.${system}.default

            # rust
            llvmPackages.clangNoLibcxx
            lldb
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer
            cargo-edit

            # surrealdb
            surrealdb
            surrealdb-migrations
          ];
        };
      });
}
