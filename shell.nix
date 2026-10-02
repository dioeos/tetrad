{ pkgs }:
pkgs.mkShell {
  packages = with pkgs; [
    distrobox
    nixd
    nixfmt
    just
  ];
}
