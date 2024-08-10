#!/usr/bin/env bash

# Name of the tmux session
SESSION="debug"

# Create a new window in the existing tmux session
tmux new-window -n debug-window

# Split the window into two panes
tmux split-window -h

# Start QEMU with monitor enabled in the first pane (left pane)
tmux send-keys -t debug-window.0 "qemu-system-i386 -display none -s -S -monitor stdio -hda bin/os.bin" C-m

# Start GDB in the second pane (right pane) and connect to QEMU
tmux send-keys -t debug-window.1 "rust-gdb" C-m
tmux send-keys -t debug-window.1 "target remote localhost:1234" C-m
tmux send-keys -t debug-window.1 "set confirm off" C-m
tmux send-keys -t debug-window.1 "set output-radix 16" C-m
tmux send-keys -t debug-window.1 "add-symbol-file build/kernelfull.o 0x101000" C-m
tmux send-keys -t debug-window.1 "break kmain" C-m

# Set up hooks to kill the session if either program exits
tmux set-option -t debug-window remain-on-exit off
tmux set-hook -t debug-window pane-died "kill-window"
