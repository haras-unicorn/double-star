{
  description = "double-star";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/release-24.05";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # versioning
            git

            # scripts
            just
            nushell

            # nix
            nil
            nixpkgs-fmt

            # markdown
            marksman
            markdownlint-cli
            nodePackages.markdown-link-check

            # spelling
            nodePackages.cspell

            # misc
            nodePackages.prettier
            nodePackages.yaml-language-server
            nodePackages.vscode-langservers-extracted
            taplo
          ];
        };
      });
}
