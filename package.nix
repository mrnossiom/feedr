{ lib

, rustPlatform
, gitignore
, htmx-src

, tailwindcss
, sqlite
}:

let
  inherit (gitignore.lib) gitignoreSource;

  src = gitignoreSource ./.;
  cargoTOML = lib.importTOML "${src}/Cargo.toml";
in
rustPlatform.buildRustPackage {
  pname = cargoTOML.package.name;
  version = cargoTOML.package.version;

  inherit src;

  cargoLock = { lockFile = "${src}/Cargo.lock"; };

  nativeBuildInputs = [ ];
  buildInputs = [
    sqlite
  ];

  postUnpack = ''
    pushd source
    cp -vf ${htmx-src} static/js/htmx.min.js
    ${lib.getExe tailwindcss} --input input.css --output static/styles/tw.css
    popd
  '';

  meta = {
    inherit (cargoTOML.package) description homepage license;
    maintainers = cargoTOML.package.authors;
    mainProgram = "feedr-server";
  };
}
