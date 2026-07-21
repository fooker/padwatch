{ rustPlatform
, pkg-config
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "padwatch";
  version = "0.1";

  src = ./.;

  cargoHash = "sha256-ee9+TleGL0IV0Eca+YAxldtYpZDNtK1koZPtzdiXiBI=";

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
}
