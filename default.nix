{ rustPlatform
, pkg-config
, openssl
, sqlite
}:

rustPlatform.buildRustPackage {
  pname = "padwatch";
  version = "0.1";

  src = ./.;

  cargoHash = "sha256-qTimnlfuiZcRrmT7F/08lnq28gfjzU8tdbkXotKVNUo=";

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
    sqlite.dev
  ];
}
