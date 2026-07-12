{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  androidTargets = [
    "aarch64-linux-android"
    "x86_64-linux-android"
  ];
in
{
  env.GREET = "Relay";

  # --- REMOVED THE MANUAL ANDROID_HOME / NDK_HOME LINES FROM HERE ---

  packages = [
    pkgs.pkg-config
    pkgs.zlib
    pkgs.dioxus-cli
    pkgs.openssl
    pkgs.android-tools

    # Linux Desktop Dev Dependencies
    pkgs.glib
    pkgs.gdk-pixbuf
    pkgs.cairo
    pkgs.pango
    pkgs.atk
    pkgs.gtk3
    pkgs.libsoup_3
    pkgs.webkitgtk_4_1
    pkgs.xdotool

    pkgs.tailwindcss-language-server
    pkgs.vscode-css-languageserver
    pkgs.yaml-language-server

    pkgs.cargo-ndk
  ];

  languages.rust = {
    enable = true;
    channel = "stable"; # Fixes the cross-compilation target issue
    targets = androidTargets;
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  # Enable Android development tooling (devenv handles the env variables here)
  android = {
    enable = true;
    platforms.version = [
      "35"
      "34"
    ];
    systemImages.enable = true;
    emulator = {
      enable = true;
    };
    ndk.enable = true;
  };

  enterShell = ''
    echo "Welcome to Relay!"
    alias emulator="env -u LD_LIBRARY_PATH emulator"
  '';

}
