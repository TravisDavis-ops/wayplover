{pkgs? import <nixos>{}}:
pkgs.mkShell {
  buildInputs = [
    pkgs.sqlite
    pkgs.glibc
  ];
  packages = [ pkgs.rustup ];
  inputsFrom = with pkgs; [alsaLib alsaUtils alsaTools alsaPlugins speechd];
  shellHook = '' echo "Wayplover"'';
}
