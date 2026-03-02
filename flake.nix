{
  description = "brokenlinks development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
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