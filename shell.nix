{ pkgs }:
pkgs.mkShell {
  packages = with pkgs; [
    distrobox
    nixd
    nixfmt
    just

    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
  ];
}
