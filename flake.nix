{
  description = "Mazrine's sick audio synthesis project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Get the latest stable Rust
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        # Audio libraries that actually work
        audioLibs = with pkgs; [
          alsa-lib
          alsa-lib.dev
          pkg-config
          udev
          vulkan-loader
          # For PulseAudio support
          libpulseaudio
          # For JACK support (because why not be extra)
          libjack2
          # Core audio stuff
          pipewire
          pipewire.dev
        ];

        # Build inputs for development
        buildInputs = with pkgs; [
          rustToolchain
          # Audio development libraries
          alsa-lib
          alsa-lib.dev
          pkg-config
          udev
          libpulseaudio
          libjack2
          pipewire
          pipewire.dev
          # General development stuff
          llvmPackages.bintools
          gcc
        ];

        # Native build inputs (for pkg-config and such)
        nativeBuildInputs = with pkgs; [
          pkg-config
          rustToolchain
        ];

        # Environment variables to make everything work
        shellHook = ''
          echo "🎵 Welcome to Mazrine's FunDSP Audio Synthesis Development Environment 🎵"
          echo "Now powered by the superior FunDSP library!"
          
          # Set up pkg-config paths for audio libs
          export PKG_CONFIG_PATH="${pkgs.alsa-lib.dev}/lib/pkgconfig:${pkgs.libpulseaudio.dev}/lib/pkgconfig:${pkgs.libjack2}/lib/pkgconfig:$PKG_CONFIG_PATH"
          
          # Make sure we can find the ALSA libraries
          export ALSA_PCM_CARD=default
          export ALSA_PCM_DEVICE=0
          
          # Rust environment
          export RUST_BACKTRACE=1
          export RUST_LOG=debug
          
          echo "PKG_CONFIG_PATH: $PKG_CONFIG_PATH"
          echo "Ready to make some noise! 🔊"
        '';

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs shellHook;
          
          # Library paths for runtime
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath audioLibs;
          
          # Additional environment variables
          ALSA_LIBS = "${pkgs.alsa-lib}/lib";
          ALSA_CFLAGS = "-I${pkgs.alsa-lib.dev}/include";
        };

        # Optional: if you want to build this as a package later
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "mazrine-audio-synth";
          version = "0.1.0";
          
          src = ./.;
          
          cargoHash = ""; # You'll need to fill this in later when packaging
          
          inherit nativeBuildInputs;
          buildInputs = audioLibs;
          
          # Make sure pkg-config can find everything
          PKG_CONFIG_PATH = "${pkgs.alsa-lib.dev}/lib/pkgconfig:${pkgs.libpulseaudio.dev}/lib/pkgconfig";
        };
      });
}
