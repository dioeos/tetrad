{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    wrapGAppsHook4
  ];

  buildInputs = with pkgs; [
    webkitgtk_4_1
    librsvg
  ];

  packages = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer

    nixd
    nixfmt

    just

    nodejs
    pnpm

    typescript
    typescript-language-server
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
}
