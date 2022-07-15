# Genshin Force FPS

This is almost a RIIR(rewrite it in Rust) for [genshin-fps-unlock](https://github.com/34736384/genshin-fps-unlock) but without GUI.

## Features
- Unlock the 30/60 fps limit in game, you can force any fps as you want
- CLI, i.e. no overhead
- Cross build

## Usage

```
Genshin Force FPS

USAGE:
  genshin-force-fps.exe [OPTIONS] [GAME_ARGS]
OPTIONS:
  -h, --help                Prints help information
  -n, --no-disable-vsync    Don't forcibly disable VSync
  -f, --fps NUMBER          Force game FPS, defaults to 120
  -o, --open PATH           Path to GenshinImpact.exe/YuanShen.exe
  -c, --cwd PATH            Path to current working dir
ARGS:
  [GAME_ARGS]               Arguments passing to game executable
EXAMPLE:
  genshin-force-fps.exe -f 120 -o C:\path\to\GenshinImpact.exe
```

Create a file shortcut with arguments appended to target path or launch a terminal to specify the arguments.

After launch, the tool will first start the game and sniffing the memory addresses of fps and vsync values, then monitoring those values using `ReadProcessMemory` and force those values using `WriteProcessMemory` respectively if not equal to what user specified at a 1 second intervals.

## Cross Build on Linux

### Generic

Install `mingw-w64-gcc` and follow instructs in https://wiki.archlinux.org/title/Rust#Windows to setup build environment.

```bash
$ cargo build --target x86_64-pc-windows-gnu
$ ls ./target/x86_64-pc-windows-gnu/*/*.exe
```

### Nix

1. Follow https://nixos.org/download.html#download-nix to setup Nix environment or install `nix` from your package manager
2. Enable Nix flakes experimental features, see https://nixos.wiki/wiki/Flakes

```bash
$ nix build
$ # or in fully qualified path
$ nix build .#packages.x86_64-linux.default
$ ls ./result/bin
```

### Troubleshooting

#### Game crashes on event screens

Change current working dir to somewhere other than parent dir of game executable.
