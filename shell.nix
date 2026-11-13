{config ? { config = { allowUnfree = true; }; }
, sources ? import ./npins
, pkgs ? import sources.nixpkgs config
}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    clippy
    rustfmt
    rust-analyzer
    nodejs
    cargo-watch
    git
    python3
    pkg-config
    jq
    treefmt
    nix-output-monitor
    just
    jujutsu
    claude-code
    tmux
    asciinema
    capnproto
    nixd
    nil
    mold
  ];

  # Let cargo find native libs
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
}
