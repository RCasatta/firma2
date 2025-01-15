{
  description = "Build Raspberry PI 4 image";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    multiqr_input = {
      url = "github:RCasatta/multiqr";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    firma2_input = {
      url = "github:RCasatta/firma2";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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
              boot.blacklistedKernelModules = [ "bluetooth" ];
              services.tlp.setting.DEVICES_TO_DISABLE_ON_STARTUP = "bluetooth";

              environment.etc."prompt-init.md".text = builtins.readFile ./prompt-init.md;
              environment.shellInit = ''
                cat /etc/prompt-init.md
              '';


              environment.sessionVariables = {
                HISTFILE = "/dev/null"; # commands are saved during the session in memory, but not across reboots
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
