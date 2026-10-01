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
          source = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "sketchpad-rs";
            version = "0.1.0";
            src = source;
            inherit cargoLock;

            nativeBuildInputs = with pkgs; [
              makeWrapper
              pkg-config
            ];
            buildInputs = [ pkgs.alsa-lib ];
            cargoBuildFlags = [ "--bin" "sketchpad_rs" ];
            doCheck = false;

            postFixup = ''
              wrapProgram "$out/bin/sketchpad_rs" \
                --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath (runtimeLibraries pkgs)}" \
                --prefix PATH : "${pkgs.lib.makeBinPath [ pkgs.fontconfig ]}"
            '';
          };

          gsp-probe = pkgs.rustPlatform.buildRustPackage {
            pname = "gsp-probe";
            version = "0.1.0";
            src = source;
            inherit cargoLock;
            cargoBuildFlags = [ "--no-default-features" "--bin" "gsp_probe" ];
            doCheck = false;
          };
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
            ];

            shellHook = ''
              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (runtimeLibraries pkgs)}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            '';
          };
        });
    };
}
