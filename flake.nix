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
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "sketchpad-rs";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

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
        });

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/sketchpad_rs";
          meta.description = "Open Sketchpad RS or launch its built-in geometry demo";
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
