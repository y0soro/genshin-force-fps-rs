# Genshin Force FPS

This is almost a RIIR(rewrite it in Rust) for [genshin-fps-unlock](https://github.com/34736384/genshin-fps-unlock) but without GUI.

## Features
- Unlock the 30/60 FPS limit in game, you can force any frame rate limit as you want
- CLI, i.e. no overhead
- Cross build

## Usage

```
Genshin Force FPS

USAGE:
  genshin-force-fps.exe [OPTIONS] -- [GAME_ARGS]
OPTIONS:
  -h, --help                Prints help information
      --hdr                 Force enable HDR support
  -f, --fps NUMBER          Force game FPS, defaults to 120
  -c, --cwd PATH            Path to working dir that game process runs on
  -o, --open PATH           Path to GenshinImpact.exe/YuanShen.exe, can be
                            omitted if it's installed on default location (C:)
ARGS:
  [GAME_ARGS]               Unity player arguments passing to game executable,
                            https://docs.unity3d.com/Manual/PlayerCommandLineArguments.html
EXAMPLE:
  # Force FPS to 120 and specify game path
  genshin-force-fps.exe -f 120 -o C:\path\to\GenshinImpact.exe
  # Force FPS to 144 and append Unity cli arguments, assuming the game was
  # installed on default location
  genshin-force-fps.exe -f 144 -- -screen-width 1600 -screen-height 900 -screen-fullscreen 0
```

The option `-o/--open` can be omitted if the game was installed on "C:\Program Files\Genshin Impact\Genshin Impact Game\".

After launching, the tool will first start the game and sniffing the memory addresses of fps value, then monitor those values using `ReadProcessMemory` and force them using `WriteProcessMemory` if not equal to what user specified at 1 second interval respectively .

### Windows

Create a file shortcut with arguments appended to target path or launch a terminal to specify the arguments. Or use batch script.

If the game was installed on default location and you are fine with default 120 fps setting, then just double click the "genshin-force-fps.exe".

### Lutris/Linux

Change game executable path to path of genshin-force-fps.exe, and specifying the game path with option `-o/--open` instead. For example,

- Executable: `/path/to/genshin-force-fps.exe`
- Arguments: `-f 144 -o 'C:\\Program Files\Genshin Impact\Genshin Impact Game\GenshinImpact.exe'`

The game path has to be Windows path in current WINEPREFIX environment instead of Unix path on host machine since this tool is still a Windows program.

## Cross Build on Linux

### Generic

Install `mingw-w64-gcc` and follow instructions in https://wiki.archlinux.org/title/Rust#Windows to setup build environment.

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
$ nix build ".#packages.x86_64-linux.default"
$ ls ./result/bin
```

## Troubleshooting

### Game crashes on event screen

Change current working dir to somewhere other than parent dir of game executable.
