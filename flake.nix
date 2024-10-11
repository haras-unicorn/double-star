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

        # TODO: use actual user driver
        nvidia_driver = (import <nixpkgs> {
          inherit system;
          config = { allowUnfree = true; };
        }).linuxPackages.nvidia_x11_production;
      in
      {
        packages.${system} = rec {
          default = double-star;
          nebulon = rustPkgs.workspace.nebulon { };
          double-star = rustPkgs.workspace.double-star { };
          orbitus = rustPkgs.workspace.orbitus { };
        };

        devShells.default = pkgs.mkShell rec {
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          CUDA_PATH = "${pkgs.cudatoolkit}";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          RUST_BACKTRACE = "full";

          DOUBLE_STAR_DB_USER = "double_star";
          DOUBLE_STAR_DB_PASS = "double_star";

          NEBULON_USER = "double_star";
          NEBULON_PASS = "double_star";

          QUALIFIER = "xyz";
          ORGANIZATION = "haras-unicorn";

          buildInputs = with pkgs; [
            # nebulon
            pkg-config
            # FIXME: breaks curl for nix
            # openssl

            # double-star
            protobuf
            cudatoolkit
            nvidia_driver

            # orbitus
            libxkbcommon
            libGL
            wayland
          ];

          packages = with pkgs; [
            # versioning
            git

            # scripts
            just
            nushell

            # spelling
            nodePackages.cspell

            # tools
            jq
            fd

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
            cargo-geiger

            # surrealdb
            surrealdb
            surrealdb-migrations
          ];

          shellHook = ''
            db="$(git rev-parse --show-toplevel)/scripts/db.nu"

            docker compose up -d
        
            DOUBLE_STAR_DB_HOST="$($db host)"
            export DOUBLE_STAR_DB_HOST
            echo "DOUBLE_STAR_DB_HOST is set to $DOUBLE_STAR_DB_HOST"

            DOUBLE_STAR_DB_PORT="$($db port)"
            export DOUBLE_STAR_DB_PORT
            echo "DOUBLE_STAR_DB_PORT is set to $DOUBLE_STAR_DB_PORT"

            NEBULON_HOST="$($db host)"
            export NEBULON_HOST
            echo "NEBULON_HOST is set to $NEBULON_HOST"

            NEBULON_PORT="$($db port)"
            export NEBULON_PORT
            echo "NEBULON_PORT is set to $NEBULON_PORT"

            $db isready
          '';
        };
      });
}
