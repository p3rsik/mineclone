{
  # mostly copied from jujutsu (https://github.com/martinvonz/jj)
  description = "Mineclone - my minecraft clone with features I always wanted";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }: {
    overlays.default = (final: prev: {
      mineclone = self.packages.${final.system}.mineclone;
    });
  } //
  (flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      };

      filterSrc = src: regexes:
        pkgs.lib.cleanSourceWith {
          inherit src;
          filter = path: type:
            let
              relPath = pkgs.lib.removePrefix (toString src + "/") (toString path);
            in
            pkgs.lib.all (re: builtins.match re relPath == null) regexes;
        };

      rust-version = pkgs.rust-bin.stable."1.76.0".default;

      ourRustPlatform = pkgs.makeRustPlatform {
        rustc = rust-version;
        cargo = rust-version;
      };

      # these are needed in both devShell and buildInputs
      darwinDeps = with pkgs; lib.optionals stdenv.isDarwin [ ];
    in
    {
      packages = {
        mineclone = ourRustPlatform.buildRustPackage {
          pname = "mineclone";
          version = "unstable-${self.shortRev or "dirty"}";
          src = filterSrc ./. [
            ".*\\.nix$"
            "^.jj/"
            "^flake\\.lock$"
            "^target/"
          ];

          cargoLock.lockFile = ./Cargo.lock;
          useNextest = true;

          nativeBuildInputs = [ ];
          buildInputs = [ ] ++ darwinDeps;

          # makes no sense in a nix package
          CARGO_INCREMENTAL = "0";

          preCheck = "export RUST_BACKTRACE=1";
          # postInstall = ''
          #   tailwindcss -i styles/tailwind.css -o assets/main.css --minify
          # '';

          # for clap apps
          # postInstall = ''
          #   $out/bin/mineclone util mangen > ./mineclone.1
          #   installManPage ./mineclone.1
          #
          #   installShellCompletion --cmd mineclone \
          #     --bash <($out/bin/mineclone util completion --bash) \
          #     --fish <($out/bin/mineclone util completion --fish) \
          #     --zsh  <($out/bin/mineclone util completion --zsh)
          # '';
        };
        default = self.packages.${system}.mineclone;
      };
      apps.default = {
        type = "app";
        program = "${self.packages.${system}.mineclone}/bin/mineclone";
      };
      devShells.default = pkgs.mkShell rec {
        buildInputs = with pkgs; [
          # Should be before rust?.
          (rust-bin.selectLatestNightlyWith (toolchain: toolchain.rustfmt))

          # Using the minimal profile with explicit "clippy" extension to avoid
          # two versions of rustfmt
          (rust-version.override {
            extensions = [
              "rust-src" # for rust-analyzer
              "clippy"
            ];
          })

          # Make sure rust-analyzer is present
          rust-analyzer

          cargo-nextest
          cargo-watch
          # cargo-insta
          # cargo-deny

          pkg-config
          udev
          alsa-lib

          vulkan-loader

          xorg.libXi
          xorg.libXrandr
          xorg.libXcursor
          xorg.libX11
          libxkbcommon
        ] ++ darwinDeps;

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

        shellHook = ''
          export RUST_BACKTRACE=1
        '';
      };
    }));
}
