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
  gstreamerPlugins = with pkgs.gst_all_1; [
    gstreamer
    gst-plugins-base
    gst-plugins-good
    gst-plugins-bad
    gst-plugins-ugly
    gst-libav
  ];
in
{
  env.GREET = "Relay";
  env.GST_PLUGIN_SYSTEM_PATH_1_0 =
    lib.makeSearchPathOutput "lib" "lib/gstreamer-1.0"
      gstreamerPlugins;
  env.GDK_PIXBUF_MODULE_FILE = "${pkgs.librsvg.out}/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache";
  env.XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:${pkgs.adwaita-icon-theme}/share:${pkgs.shared-mime-info}/share";
  android = {
    enable = true;
    platforms.version = [
      "35"
      "34"
    ];
    systemImages.enable = true;
    ndk.enable = true;
  };

  packages = [
    pkgs.pkg-config
    pkgs.zlib
    pkgs.dioxus-cli
    pkgs.openssl
    pkgs.android-tools
    pkgs.librsvg
    pkgs.adwaita-icon-theme
    pkgs.shared-mime-info
    pkgs.gsettings-desktop-schemas

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
  ]
  ++ gstreamerPlugins;

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

  enterShell = ''
    export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
    mkdir -p .android/tmp
    echo "Welcome to Relay!"
    alias emulator="env -u LD_LIBRARY_PATH emulator"
  '';

}
