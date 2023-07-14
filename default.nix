{ lib
, fetchFromGitHub
, rustPlatform
, pkg-config
, openssl
}:

rustPlatform.buildRustPackage rec {
  pname = "padwatch";
  version = "0.1";

  src = ./.;

  cargoHash = "sha256-4rvq3Y9ohZJJtrCil4qokrPPyz9r8SLfBy44xmrT9OM=";

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
}
