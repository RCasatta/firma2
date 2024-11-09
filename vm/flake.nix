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

              environment.etc."readme.md".text = ''
                # Welcome to firma, an offline PSBT signer.

                Available commands: deriva, firma, multiqr, jq, vim, age.

                Once time setup by creating an password encrypted file `SEED.age` file with: `cat - | age -e -p -a > SEED.age` and inputting the `SEED` (end with Enter Ctrl-D).

                View standard descriptors, or ask a custom derivation with `cat SEED.age | age -d | deriva`

                Initialize the `DESCRIPTOR` env var with default pay to taproot key spend `export DESCRIPTOR=$(cat SEED.age | age -d | deriva | jq -r .default)`

                Sign a psbt with `cat SEED.age | age -d | firma psbt-file`
              '';
              environment.shellInit = ''
                cat /etc/readme.md
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
