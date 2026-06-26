{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
  packages = [
    pkgs.pkg-config
    pkgs.zlib
    pkgs.dioxus-cli
    pkgs.openssl
    pkgs.glib
    pkgs.gdk-pixbuf
    pkgs.cairo
    pkgs.pango
    pkgs.atk
    pkgs.gtk3
    pkgs.libsoup_3
    pkgs.webkitgtk_4_1
    pkgs.xdotool
  ];

  # https://devenv.sh/languages/
  languages = {
    rust = {
      enable = true;
      # Enables the rust-analyzer component for IDE support
      components = [
        "rustc"
        "cargo"
        "clippy"
        "rustfmt"
        "rust-analyzer"
      ];
    };

  };

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    export NIX_CFLAGS_COMPILE="-fno-omit-frame-pointer $NIX_CFLAGS_COMPILE"
    echo "hello from $GREET (with Rust, CMake, and Valgrind ready!)"
  '';

  # https://devenv.sh/basics/
  enterShell = ''
    hello welcome to relay, have fun coding!
    cargo --version
    rustc --version
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    cargo test
  '';
}
