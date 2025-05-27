{ rustPlatform
, pkg-config
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "padwatch";
  version = "0.1";

  src = ./.;

  cargoHash = "sha256-2ZRkJyQspKwwqf1mKxun1a5DuQLu7epyIsj3oyliUqc=";

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
}
