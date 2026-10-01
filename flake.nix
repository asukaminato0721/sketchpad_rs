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
          # Keep test-only corpus changes out of production package rebuilds.
          nativeTemplates = with pkgs.lib.fileset; unions [
            ./tests/fixtures/coordinate_measure_base.gsp
            ./tests/fixtures/coordinate_measure_menu257.gsp
            ./tests/fixtures/coordinate_measure_menu274.gsp
            ./tests/fixtures/coordinate_measure_menu291.gsp
            ./tests/fixtures/coordinate_measure_menu308.gsp
            ./tests/fixtures/function_base_frame.gsp
            ./tests/fixtures/function_derivative_square_original.gsp
            ./tests/fixtures/function_plot_identity.gsp
            ./tests/fixtures/function_plot_square.gsp
            ./tests/fixtures/function_reference_controls_native_created.gsp
            ./tests/fixtures/image_original_cropped.gsp
            ./tests/fixtures/image_port_tiny.gsp
            ./tests/fixtures/numeric_parameter_table_all_unnamed_native_created.gsp
            ./tests/fixtures/parameter_calculation_chain_native.gsp
            ./tests/fixtures/parametric_polar_original.gsp
            ./tests/fixtures/parametric_rectangular_original.gsp
            ./tests/fixtures/plot_points_table_polar_original.gsp
            ./tests/fixtures/plot_points_table_single_original.gsp
            ./tests/fixtures/point_on_arc_value_native.gsp
            ./tests/fixtures/point_on_circle_arc_value_native.gsp
            ./tests/fixtures/table_calculate_2_plus_3.gsp
            ./tests/fixtures/table_parameter_default.gsp
            ./tests/fixtures/table_parameter_speed_original.gsp
            ./tests/fixtures/table_parameter_tabulated.gsp
            ./tests/fixtures/table_two_columns_original.gsp
            ./tests/fixtures/table_two_parameters_original.gsp
            ./tests/fixtures/text_object_generated_Hello.gsp
            ./tests/fixtures/text_style_bold_off.gsp
            ./tests/fixtures/text_style_font_cjk.gsp
            ./tests/fixtures/text_style_italic_on.gsp
            ./tests/fixtures/text_style_larger.gsp
            ./tests/fixtures/text_style_underline_on.gsp
            ./tests/fixtures/value_equation_circle_original.gsp
            ./tests/fixtures/value_equation_line_original.gsp
            ./tests/fixtures/value_equation_point_value_original.gsp
          ];
          source = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./src
              ./crates
              ./assets
              nativeTemplates
            ];
          };
          commonPackage = {
            version = "0.1.0";
            src = source;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = false;
          };
          desktopItem = pkgs.makeDesktopItem {
            name = "sketchpad-rs";
            desktopName = "Sketchpad RS";
            comment = "Open and edit Geometer's Sketchpad documents";
            exec = "sketchpad_rs %f";
            icon = "applications-graphics";
            categories = [ "Education" "Math" ];
            mimeTypes = [ "application/x-geometer-sketchpad" ];
            actions.Demo = {
              name = "Open Built-in Demo";
              exec = "sketchpad_rs --demo";
            };
          };
          mimeInfo = pkgs.writeTextFile {
            name = "sketchpad-rs-mime";
            destination = "/share/mime/packages/sketchpad-rs.xml";
            text = ''
              <?xml version="1.0" encoding="UTF-8"?>
              <mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
                <mime-type type="application/x-geometer-sketchpad">
                  <comment>Geometer's Sketchpad document</comment>
                  <glob pattern="*.gsp"/>
                </mime-type>
              </mime-info>
            '';
          };
          guiBinary = pkgs.rustPlatform.buildRustPackage (commonPackage // {
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
        in
        {
          default = pkgs.symlinkJoin {
            name = "sketchpad-rs-0.1.0";
            paths = [
              guiBinary
              desktopItem
              mimeInfo
            ];
            meta = {
              description = "Linux editor for Geometer's Sketchpad documents";
              mainProgram = "sketchpad_rs";
            };
          };

          gsp-probe = pkgs.rustPlatform.buildRustPackage (commonPackage // {
            pname = "gsp-probe";
            cargoBuildFlags = [ "--no-default-features" "--bin" "gsp_probe" ];
            meta = {
              description = "Headless tools for Geometer's Sketchpad documents";
              mainProgram = "gsp_probe";
            };
          });
        });

      apps = forAllSystems (system:
        let
          packages = self.packages.${system};
          pkgs = packagesFor system;
          demoLauncher = pkgs.writeShellScript "sketchpad-rs-demo" ''
            exec ${packages.default}/bin/sketchpad_rs --demo
          '';
        in
        {
          default = {
            type = "app";
            program = "${packages.default}/bin/sketchpad_rs";
            meta.description = "Open the Sketchpad RS geometry editor";
          };
          gsp-probe = {
            type = "app";
            program = "${packages.gsp-probe}/bin/gsp_probe";
            meta.description = "Inspect, verify, convert, and generate Sketchpad documents";
          };
          demo = {
            type = "app";
            program = "${demoLauncher}";
            meta.description = "Launch the built-in live geometry demonstration";
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
