{
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system: let
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    packages.default = pkgs.callPackage ./default.nix { };

    devShells.default = pkgs.mkShell {
      inputsFrom = [ self.packages.${system}.default ];
    };
  });
}

