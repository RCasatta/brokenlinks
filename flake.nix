{
  description = "brokenlinks development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage rec {
        pname = "brokenlinks";
        version = "0.1.1";
        src = ./.;
        cargoLock = { lockFile = ./Cargo.lock; };
        nativeBuildInputs = with pkgs; [ rust-bin.stable.latest.default pkg-config ];
        buildInputs = with pkgs; [ openssl ];
      };

      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustup
          openssl
          pkg-config
        ];

        shellHook = ''
          export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
          export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
          export PKG_CONFIG_PATH="${pkgs.openssl.out}/lib/pkgconfig"
        '';
      };
    };
}