# Genshin Force FPS

This is almost a RIIR(rewrite it in Rust) for [genshin-fps-unlock](https://github.com/34736384/genshin-fps-unlock) but without GUI.

## Features
- Unlock the 30/60 fps limit in game, you can force any fps as you want.
- CLI, i.e. no overhead
- Cross build

## Usage

```
Genshin Force FPS

USAGE:
  genshin-force-fps.exe [OPTIONS] [GAME_ARGS]
OPTIONS:
  -h, --help                Prints help information
  -n, --no-disable-async    Don't forcibly disable VSync
  -f, --fps NUMBER          Force game FPS, defaults to 120
  -o, --open PATH           Path to GenshinImpact.exe/YuanShen.exe
ARGS:
  [GAME_ARGS]               Arguments passing to game executable
EXAMPLE:
  genshin-force-fps.exe -f 120 -o C:\path\to\GenshinImpact.exe
```

## Cross Build on Linux

Install `mingw-w64-gcc` and follow instructs in https://wiki.archlinux.org/title/Rust#Windows to setup build environment.

```bash
$ cargo build --target x86_64-pc-windows-gnu
$ ls ./target/x86_64-pc-windows-gnu/*/*.exe
```
