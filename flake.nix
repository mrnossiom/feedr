{
  inputs = {
    # nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    process-compose.url = "github:Platonic-Systems/process-compose-flake";

    services-flake.url = "github:juspay/services-flake";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    gitignore.url = "github:hercules-ci/gitignore.nix";
    gitignore.inputs.nixpkgs.follows = "nixpkgs";

    # Sources
    htmx-src.url = "https://unpkg.com/htmx.org@2.0.4/dist/htmx.min.js";
    htmx-src.flake = false;
  };

  outputs = { self, nixpkgs, process-compose, services-flake, rust-overlay, gitignore, htmx-src }:
    let
      inherit (nixpkgs.lib) genAttrs;

      forAllSystems = genAttrs [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
      forAllPkgs = function: forAllSystems (system: function pkgs.${system});

      pkgs = forAllSystems (system: (import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      }));

      services = {
        services.postgres."pg1" = {
          enable = true;
          createDatabase = false;
        };
      };

      servicesOutput = forAllPkgs (pkgs:
        let
          pc-lib = import process-compose.lib { inherit pkgs; };
          eval = pc-lib.evalModules { modules = [ services-flake.processComposeModules.default services ]; };
        in
        eval.config);
    in
    {
      formatter = forAllPkgs (pkgs: pkgs.nixpkgs-fmt);

      packages = forAllPkgs (pkgs: rec {
        default = feedr;
        feedr = pkgs.callPackage ./package.nix { inherit gitignore htmx-src; };
        dev-services = servicesOutput.${pkgs.system}.outputs.package;
      });

      devShells = forAllPkgs (pkgs:
        with pkgs.lib;
        let
          file-rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rust-toolchain = file-rust-toolchain.override { extensions = [ "rust-analyzer" ]; };
        in
        {
          default = pkgs.mkShell rec {
            inputsFrom = [
              servicesOutput.${pkgs.system}.services.outputs.devShell
            ];

            nativeBuildInputs = with pkgs; [
              cargo-expand
              just
              pkg-config
              rust-toolchain
              watchexec

              diesel-cli
              postgresql
              tailwindcss
            ];

            buildInputs = with pkgs; [
              postgresql
            ];

            shellHook = ''
              echo [flake] Copying htmx,fonts source...
              mkdir -p static/js
              cp -f ${htmx-src} static/js/htmx.min.js
            '';
            # cp -rf ${lucide-src}/*.svg appview/pages/static/icons/
            # cp -f ${inter-fonts-src}/web/InterVariable*.woff2 appview/pages/static/fonts/
            # cp -f ${inter-fonts-src}/web/InterDisplay*.woff2 appview/pages/static/fonts/
            # cp -f ${ibm-plex-mono-src}/fonts/complete/woff2/IBMPlexMono-Regular.woff2 appview/pages/static/fonts/

            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
            LD_LIBRARY_PATH = makeLibraryPath buildInputs;

            # TODO: remove watchexec when env filter PR is merged
            RUST_LOG = "info,feedr_server=debug,tower_http=debug,watchexec=error";
            # DATABASE_URL = "file:./data.sqlite";
            DATABASE_URL = "postgres://localhost/feedr";
          };
        });
    };
}
