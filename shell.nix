{ pkgs }:
pkgs.mkShell {
  packages = with pkgs; [
    distrobox
    nixd
    nixfmt
    just
    gnumake
    gcc

    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer

    sqlx-cli
    sqlite
  ];
}
