{
  description = "Tetrad flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      tetrad-server-pkg = pkgs.rustPlatform.buildRustPackage {
        pname = "tetrad-server";
        version = "0.1.0";

        src = ./.;
        buildAndTestSubdir = "apps/tetrad-server";
        cargoLock.lockFile = ./Cargo.lock;
      };

      tetrad-tauri-pkg = pkgs.rustPlatform.buildRustPackage (finalAttrs: {
        pname = "tetrad-client";
        version = "0.1.0";

        src = ./.;
        buildAndTestSubdir = "apps/tetrad-client/src-tauri";
        cargoLock.lockFile = ./Cargo.lock;

        pnpmRoot = "apps/tetrad-client";

        pnpmDeps = pkgs.pnpm.fetchDeps {
          inherit (finalAttrs) pname version;
          src = ./apps/tetrad-client;
          fetcherVersion = 3;
          hash = pkgs.lib.fakeHash;
        };

        nativeBuildInputs =
          with pkgs;
          [
            cargo-tauri.hook
            nodejs
            pnpm
            pnpm.configHook
            pkg-config
          ]
          ++ lib.optionals stdenv.hostPlatform.isLinux [ wrapGAppsHook4 ];

        buildInputs = pkgs.lib.optionals pkgs.hostPlatform.isLinux (
          with pkgs;
          [
            glib-networking
            openssl
            webkitgtk_4_1
            librsvg
          ]
        );
      });
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        inputsFrom = [
          tetrad-server-pkg
          tetrad-tauri-pkg
        ];

        #@NOTE: `pkgs.rustPlatform.buildRustPackage` includes most of the standard
        #       Rust toolchain (cargo + rustc), but does not include other common useful tooling
        packages = with pkgs; [
          rustfmt
          clippy
          rust-analyzer

          nixd
          nixfmt
          just
          sqlx-cli
          sqlite
        ];

        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

        shellHook = ''
          export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH:$XDG_DATA_DIRS"
          echo "Entered Tetrad shell..."
          cargo --version
          rustc --version
          node --version
          pnpm --version
          tsc --version
        '';
      };

      packages.${system} = {
        server = tetrad-server-pkg;
        client = tetrad-tauri-pkg;
      };
    };
}
