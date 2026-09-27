{
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system: let
    pkgs = nixpkgs.legacyPackages.${system};

    rust-toolchain = pkgs.symlinkJoin {
      name = "rust-toolchain";
      paths = with pkgs; [
        rustc
        cargo
        rustfmt
        clippy
      ];
      postBuild = ''
        mkdir -p $out/lib/rustlib/src/rust
        ln -s ${pkgs.rustPlatform.rustLibSrc} $out/lib/rustlib/src/rust/library
      '';
    };

  in {
    packages.default = pkgs.callPackage ./default.nix { };

    devShells.default = pkgs.mkShell {
      inputsFrom = [ self.packages.${system}.default ];

      packages = [
        rust-toolchain
      ];

      env.RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust";
    };
  });
}

