{
  description = "WayPlover - Steno for Wayland";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs"; # Pin nixpkgs for increased reproducibility
    naersk.url = "github:nmattia/naersk"; # Needed for building a rust project with nix
  };

  outputs = { self, nixpkgs, naersk }: let
    # The architecture for the build
    arch = "x86_64-linux";

    # Which package set to use
    pkgs = nixpkgs.legacyPackages.${arch};

    # The binary program
    wayplover = naersk.lib.${arch}.buildPackage {
      pname = "wayplover";
      root = ./.;
      buildInputs = with pkgs; [ sqlite ]; # TODO: possibly add alsa deps here
    };
  in {
    # Use `nix build' to build and `nix run' to run
    defaultPackage.${arch} = wayplover;

    # Protip: use direnv to load this automatically
    devShell.${arch} = pkgs.mkShell {
      buildInputs = with pkgs; [ rustc cargo figlet ];
      shellHook = ''
        figlet "WayPlover"
        git status -s -b
      '';
    };
  };
}
