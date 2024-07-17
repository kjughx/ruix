{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
    buildInputs = with pkgs; [
        gcc
        git
        eza
        gdb
        fd
        nasm
        ripgrep
        rustup
        rust-analyzer
        qemu
        zed-editor
    ];
    env = {
        SHELL="/usr/bin/fish";
    };
    shellHook = ''
        alias ls=eza
        alias ll="ls -alh --color=auto"
        alias find=fd
        alias vi=nvim
        alias vim=nvim
        exec tmux
    '';
}
