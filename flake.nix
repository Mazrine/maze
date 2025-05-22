
{
  description = "Maze DAW - Terminal-based Digital Audio Workstation";

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
        
        # Use the latest stable Rust with additional components
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };

        # Audio development dependencies
        audioDeps = with pkgs; [
          alsa-lib
          alsa-lib.dev
          pkg-config
          pipewire
          pipewire.dev
          jack2
          portaudio
        ];

        # Build dependencies
        buildDeps = with pkgs; [
          gcc
          cmake
          llvmPackages.clang
          llvmPackages.libclang
        ];

        # Development tools
        devTools = with pkgs; [
          # Audio tools for testing
          pavucontrol
          qjackctl
          audacity
          
          # General dev tools
          git
          just  # Like make but better
          bacon # Background Rust compiler
          
          # Debugging
          gdb
          valgrind
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [ rustToolchain ] ++ audioDeps ++ buildDeps ++ devTools;

          shellHook = ''
            # Set up audio development environment
            export PKG_CONFIG_PATH="${pkgs.alsa-lib.dev}/lib/pkgconfig:${pkgs.pipewire.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
            export LD_LIBRARY_PATH="${pkgs.alsa-lib}/lib:${pkgs.pipewire.lib}/lib:$LD_LIBRARY_PATH"
            
            # Rust-specific environment
            export RUST_BACKTRACE=1
            export RUST_LOG=debug
            
            # Audio-specific environment
            export ALSA_CARD=0  # Default to first audio card
            
            # Clang for bindgen (used by some audio crates)
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
            export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.glibc.dev}/include"

            echo "🎵🦀 Maze DAW Development Environment"
            echo "=================================="
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            echo "Audio: ALSA + PipeWire + JACK available"
            echo ""
            echo "Available commands:"
            echo "  cargo build          - Build the project"
            echo "  cargo run            - Run the DAW"
            echo "  cargo test           - Run tests"
            echo "  bacon                - Watch mode compilation"
            echo "  just --list          - Show available tasks"
            echo ""
            echo "Audio tools:"
            echo "  pavucontrol          - PulseAudio/PipeWire volume control"
            echo "  qjackctl             - JACK control panel"
            echo ""
          '';

          # Ensure we can find dynamic libraries at runtime
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath audioDeps;
        };

        # Optional: Package the application itself
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "maze-daw";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = buildDeps ++ [ pkgs.pkg-config ];
          buildInputs = audioDeps;

          # Set up the same environment variables for building
          PKG_CONFIG_PATH = "${pkgs.alsa-lib.dev}/lib/pkgconfig:${pkgs.pipewire.dev}/lib/pkgconfig";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          meta = with pkgs.lib; {
            description = "A terminal-based Digital Audio Workstation built in Rust";
            homepage = "https://github.com/mazrine/maze-daw"; # Update this
            license = licenses.mit; # or whatever license you choose
            maintainers = [ "mazrine" ];
            platforms = platforms.linux;
          };
        };

        # Optional: Development apps you can run with `nix run .#<name>`
        apps = {
          # Watch mode for development
          watch = {
            type = "app";
            program = toString (pkgs.writeShellScript "watch-maze-daw" ''
              ${pkgs.bacon}/bin/bacon
            '');
          };

          # Run with debug logging
          debug = {
            type = "app";
            program = toString (pkgs.writeShellScript "debug-maze-daw" ''
              export RUST_LOG=debug
              cargo run
            '');
          };
        };
      });
}
