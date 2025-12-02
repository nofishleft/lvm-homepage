{
  description = "A systemd service for displaying basic LVM stats in Homepage";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      overlay = final: prev: {
          lvm-homepage = final.callPackage ./default.nix { };
      };
    in
    {
      overlays.default = overlay;

      nixosModules.default = import ./service.nix self;

    } // flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [overlay];
        };
      in
      {
        packages = {
          lvm-homepage = pkgs.lvm-homepage;
          default = pkgs.lvm-homepage;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ pkgs.lvm-homepage ];
        };
      }
    );
}
