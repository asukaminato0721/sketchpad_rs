{
  description = "Sketchpad RS desktop editor";

  inputs.nixpkgs.url = "nixpkgs";

  outputs = { self, nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      packagesFor = system: import nixpkgs { inherit system; };
      runtimeLibraries = pkgs: with pkgs; [
        wayland
        libxkbcommon
        libglvnd
        mesa
        vulkan-loader
        libx11
        libxcursor
        libxi
        libxrandr
        alsa-lib
      ];
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = packagesFor system;
          source = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./src
              ./crates
              ./assets
              ./tests/fixtures
            ];
          };
          commonPackage = {
            version = "0.1.0";
            src = source;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = false;
          };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage (commonPackage // {
            pname = "sketchpad-rs";

            nativeBuildInputs = with pkgs; [
              makeWrapper
              pkg-config
            ];
            buildInputs = [ pkgs.alsa-lib ];
            cargoBuildFlags = [ "--bin" "sketchpad_rs" ];

            postFixup = ''
              wrapProgram "$out/bin/sketchpad_rs" \
                --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath (runtimeLibraries pkgs)}" \
                --prefix PATH : "${pkgs.lib.makeBinPath [ pkgs.fontconfig ]}"
            '';
          });

          gsp-probe = pkgs.rustPlatform.buildRustPackage (commonPackage // {
            pname = "gsp-probe";
            cargoBuildFlags = [ "--no-default-features" "--bin" "gsp_probe" ];
          });
        });

      apps = forAllSystems (system:
        let
          packages = self.packages.${system};
        in
        {
          default = {
            type = "app";
            program = "${packages.default}/bin/sketchpad_rs";
            meta.description = "Open Sketchpad RS or launch its built-in geometry demo";
          };
          gsp-probe = {
            type = "app";
            program = "${packages.gsp-probe}/bin/gsp_probe";
            meta.description = "Inspect, verify, convert, and generate Sketchpad documents";
          };
        });

      devShells = forAllSystems (system:
        let
          pkgs = packagesFor system;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              clippy
              rustfmt
              pkg-config
              fontconfig
              alsa-lib
              xvfb-run
              xdotool
              imagemagick
            ];

            shellHook = ''
              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (runtimeLibraries pkgs)}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            '';
          };
        });
    };
}
