{
  description = "Create and manage macOS, Linux, and Windows virtual machines with intuitive configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        naersk' = pkgs.callPackage naersk {
            cargo = pkgs.rust-bin.stable.latest.default;
            rustc = pkgs.rust-bin.stable.latest.default;
        };
      
      in rec {
        defaultPackage = naersk'.buildPackage {
          name = "quickemu-rs";
          src = ./.;
          buildInputs = with pkgs; [ xorg.libxcb ];
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rust-bin.stable.latest.default cargo xorg.libxcb ];
        };
      }
    );
}
