{
  description = "Create and manage macOS, Linux, and Windows virtual machines with intuitive configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};
      
      in rec {
        defaultPackage = naersk'.buildPackage {
          name = "quickemu-rs";
          src = ./.;
          buildInputs = with pkgs; [ xorg.libxcb ];
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo xorg.libxcb ];
        };
      }
    );
}
