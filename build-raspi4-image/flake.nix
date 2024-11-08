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

              users.users.root.initialHashedPassword = "$y$j9T$/29noYRT4W/22Hy4lW7B71$MNtGBgjk01Zo3LtKgFRQtwaXdv6I15oiSgGGCMkt9s2"; # =test use mkpasswd to generate
              system = {
                stateVersion = "23.05";
              };
              networking = {
                wireless.enable = false;
                useDHCP = false;
              };
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
