# LC3 VM

This is an LC3 virtual machine written in Rust, following
[this tutorial](https://justinmeiners.github.io/lc3-vm/).

This project also features a TUI terminal debugger for the VM that allows you
to step through a binary, instruction by instruction. It will let you see the
execution context and which opcodes/trapcodes are being executed by the CPU, as
well as the state of the registers at each iteration.

[![asciicast](https://asciinema.org/a/gjmyL3thLqxmffcu0tpoZPSZh.svg)](https://asciinema.org/a/gjmyL3thLqxmffcu0tpoZPSZh)

<script id="asciicast-gjmyL3thLqxmffcu0tpoZPSZh" src="https://asciinema.org/a/gjmyL3thLqxmffcu0tpoZPSZh.js" async></script>

## Compilation

Build this project with `cargo build --release` (it targets stable Rust).
