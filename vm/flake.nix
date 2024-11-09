# `nix run .#vm`
{
  description = "A virtual machine with firma2 binaries";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    firma2_input = {
      url = "github:RCasatta/firma2";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    multiqr_input = {
      url = "github:RCasatta/multiqr";
      inputs.nixpkgs.follows = "nixpkgs";
    };

  };

  outputs = { self, nixpkgs, firma2_input, multiqr_input, ... }:
    let
      system = "x86_64-linux";
      firma2 = firma2_input.packages.${system}.default;
      multiqr = multiqr_input.packages.${system}.default;

      # test is a hostname for our machine
      nixosConfigurations.firma2 = nixpkgs.lib.nixosSystem {
        inherit system;

        modules =
          [
            ({ pkgs, ... }: {
              # Let 'nixos-version --json' know about the Git revision
              # of this flake.
              system.configurationRevision = nixpkgs.lib.mkIf (self ? rev) self.rev;

              system.stateVersion = "24.05";

              services.getty.autologinUser = "root";

              environment.etc."prompt-init.md".text = builtins.readFile ./prompt-init.md;
              environment.shellInit = ''
                cat /etc/prompt-init.md
              '';

              virtualisation.vmVariant = {
                virtualisation = {
                  memorySize = 4096; # MiB.
                  cores = 1;
                  diskSize = 1000; # 1GB
                  graphics = false;
                };

              };

              networking = {
                wireless.enable = false;
                useDHCP = false;
              };

              environment.systemPackages = with pkgs; [
                vim
                jq
                age
                xterm # provide resize
              ] ++ [
                firma2
                multiqr
              ];

            })
          ];
      };
    in
    {
      vm = nixosConfigurations.firma2.config.system.build.vm;
    };
}

