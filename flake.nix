{
  description = "Crabsplit";

  # Nixpkgs / NixOS version to use.
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      overlays = [
        rust-overlay.overlays.default
        (final: prev: {
          rustToolchain =
            let
              rust = prev.rust-bin;
            in
            if builtins.pathExists ./rust-toolchain.toml then
              rust.fromRustupToolchainFile ./rust-toolchain.toml
            else if builtins.pathExists ./rust-toolchain then
              rust.fromRustupToolchainFile ./rust-toolchain
            else
              rust.stable.latest.default.override {
                extensions = [ "rust-src" "rustfmt" ];
              };
        })
      ];


      # Generate a user-friendly version number.
      version = builtins.substring 0 8 self.lastModifiedDate;

      # System types to support.
      supportedSystems = [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin" ];

      # Helper function to generate an attrset '{ x86_64-linux = f "x86_64-linux"; ... }'.
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # Nixpkgs instantiated for supported system types.
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit overlays system;  });
    in
    {

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {
          default = pkgs.mkShell rec {
            packages = with pkgs; [
              rustToolchain
              rust-analyzer
            ];

            buildInputs = with pkgs; [
              libxkbcommon
              libGL

              # WINIT_UNIX_BACKEND=wayland
              wayland

              # WINIT_UNIX_BACKEND=x11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              xorg.libX11
            ];
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          };
        });

      # Provide some binary packages for selected system types.
      packages = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {
          # The default package for 'nix build'. This makes sense if the
          # flake provides only one package or there is a clear "main"
          # package.
          default = pkgs.rustPlatform.buildRustPackage rec {
            pname = "crabsplit";
            inherit version;
            # In 'nix develop', we don't need a copy of the source tree
            # in the Nix store.
            src = ./.;
            cargoHash = "sha256-RUmzL/3g+vocNAEG0EZj1d9XgKrD8E/sxihsuDYL/7U=";

            buildInputs = with pkgs; [
              libxkbcommon
              libGL

              # WINIT_UNIX_BACKEND=wayland
              wayland

              # WINIT_UNIX_BACKEND=x11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              xorg.libX11
            ];

            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";            

            postFixup = ''
              patchelf --set-rpath ${pkgs.lib.makeLibraryPath buildInputs} $out/bin/crabsplit
            '';
          };
        });
    };
}
