{
  description = "Build Raspberry PI 4 image";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    multiqr_input.url = "github:RCasatta/multiqr";
    firma2_input.url = "github:RCasatta/firma2";
  };
  outputs = { self, nixpkgs, firma2_input, multiqr_input }:
    let
      system = "aarch64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
      firma2 = firma2_input.packages.${system}.default;
      multiqr = multiqr_input.packages.${system}.default;
      nixosConfigurations.rpi4 = nixpkgs.lib.nixosSystem {
        inherit system pkgs;
        modules = [
          "${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix"
          ({ ... }: {
            config = {
              time.timeZone = "Europe/Rome";
              i18n.defaultLocale = "it_IT.UTF-8";
              sdImage.compressImage = false;
              console.keyMap = "it";

              services.getty.autologinUser = "root";

              system = {
                stateVersion = "24.05";
              };
              networking = {
                wireless.enable = false;
                useDHCP = false;
              };
              hardware.bluetooth.enable = false;

              environment.systemPackages = with pkgs; [
                vim
                age
                jq
              ] ++ [
                firma2
                multiqr
              ];
            };
          })
        ];
      };
    in
    {
      image.rpi4 = nixosConfigurations.rpi4.config.system.build.sdImage;
    };

}
