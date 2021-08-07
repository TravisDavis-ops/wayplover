{pkgs? import <nixos>{}}:
pkgs.mkShell {
  buildInputs = [
    pkgs.sqlite
    pkgs.glibc
    pkgs.rustc
    pkgs.cargo
  ];
  packages = [ pkgs.rustup  pkgs.figlet];
  inputsFrom = with pkgs; [alsaLib alsaUtils alsaTools alsaPlugins];
  shellHook = ''
    figlet "WayPlover"
    build(){
      cargo build || cargo test
    }
    run() {
      cargo run -- -d ../plover.db -p /dev/ttyACM0
    }
    git status -s -b
  '';
}
