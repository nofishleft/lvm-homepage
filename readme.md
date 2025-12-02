# LVM Homepage

A systemd service for displaying basic LVM stats in [Homepage](https://github.com/gethomepage/homepage)


## Installation

### NixOS (Flakes)


```nix
# flake.nix
inputs = {
  ...
  lvm-homepage = {
    url = "github:nofishleft/lvm-homepage";
    inputs.nixpkgs.follows = "nixpkgs";
  };
};

outputs = { self, lvm-homepage, ... }: {
  ...
  modules = [
    lvm-homepage.nixosModules.default
    ./configuration.nix
  ];
};
```

```nix
# configuration.nix

services.lvm-homepage = {
  enable = true;
  host = "127.0.0.1"; #default
  port = "9000"; #default
  user = "lvm-homepage"; #default
  group = "lvm-homepage"; #default
};

services.homepage-dashboard = {
  enable = true;
  services = [
    {
      "LVM" = [
        {
          "Logical Volumes" = {
              icon = "";
              widget = {
                type = "customapi";
                url = "http://127.0.0.1:9000/lvs";
                refreshInterval = 30000;
                display = "dynamic-list";
                mappings = {
                  name = "value";
                  label = "name";
                };
              };
            };
          }
          {
            "Volume Groups" = {
              icon = "";
              widget = {
                type = "customapi";
                url = "http://127.0.0.1:9000/vgs";
                refreshInterval = 30000;
                display = "dynamic-list";
                mappings = {
                  name = "value";
                  label = "name";
                };
              };
            };
          }
          {
            "Physical Volumes" = {
              icon = "";
              widget = {
                type = "customapi";
                url = "http://127.0.0.1:9000/pvs";
                refreshInterval = 30000;
                display = "dynamic-list";
                mappings = {
                  name = "value";
                  label = "name";
                };
              };
            };
          };
        }
      ];
    }
  ];
};
```
