use core::time::Duration;
use std::error::Error;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::thread::sleep;

use genshin_force_fps::process::module::Module;
use genshin_force_fps::process::Process;

use pico_args::Arguments;
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{GetFileAttributesW, INVALID_FILE_ATTRIBUTES};

const HELP: &str = "\
Genshin Force FPS

USAGE:
  genshin-force-fps.exe [OPTIONS] [GAME_ARGS]
OPTIONS:
  -h, --help                Prints help information
  -n, --no-disable-async    Don't force disable VSync
  -f, --fps NUMBER          Force game FPS, defaults to 120
  -o, --open PATH           Path to GenshinImpact.exe/YuanShen.exe
ARGS:
  [GAME_ARGS]               Arguments passing to game executable
EXAMPLE:
  genshin-force-fps.exe -f 120 -o C:\\path\\to\\GenshinImpact.exe
";

const DEFAULT_GAME_PATHS: &[&'static str] = &[
    "C:\\Program Files\\Genshin Impact\\Genshin Impact Game\\GenshinImpact.exe",
    "C:\\Program Files\\Genshin Impact\\Genshin Impact Game\\YuanShen.exe",
];

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let game_path = args.opt_value_from_str::<_, String>(["-o", "--open"])?;
    let game_path = match game_path {
        Some(s) => s,
        None => {
            let res = DEFAULT_GAME_PATHS.iter().find(|i| path_exists(i));
            if let Some(&s) = res {
                s.to_owned()
            } else {
                println!("{}", HELP);
                println!("Please specify the game path with option -o");
                std::process::exit(1);
            }
        }
    };
    if !path_exists(&game_path) {
        println!("Path {} does not exist", game_path);
        std::process::exit(1);
    }

    let fps = args
        .opt_value_from_str::<_, i32>(["-f", "--fps"])?
        .unwrap_or(120);
    let fps = ::core::cmp::max(1, fps);

    let disable_vsync = args
        .opt_free_from_fn(|s| {
            if s == "-n" || s == "--no-disable-async" {
                Ok(())
            } else {
                Err("".to_owned())
            }
        })?
        .is_none();

    let game_args: Vec<String> = args
        .finish()
        .into_iter()
        .map(|i| i.to_str().unwrap().to_owned())
        .collect();

    let ps = Process::create(&game_path, &game_args.join(" "))?;
    let m = loop {
        sleep(Duration::from_millis(200));
        match ps.get_module("UnityPlayer.dll") {
            Ok(m) => break m,
            Err(s) => {
                eprintln!("{}", s);
            }
        }
        if !ps.is_active() {
            return Ok(());
        }
    };

    let p_fps = scan_fps_ptr(&ps, &m)?;
    let p_vsync = scan_vsync_ptr(&ps, &m)?;

    eprintln!("scan success: p_fps:{:?}, p:vsync{:?}", p_fps, p_vsync);
    drop(m);

    loop {
        if !ps.is_active() {
            return Ok(());
        }
        sleep(Duration::from_secs(1));

        let res = ps.read::<i32>(p_fps);
        if let Ok(v) = res {
            if v != fps && v >= 0 {
                let res = ps.write::<i32>(p_fps, &fps);
                if res.is_err() {
                    eprintln!("failed to write FPS");
                } else {
                    eprintln!("Force FPS: {} -> {}", v, fps);
                }
            }
        }

        if disable_vsync {
            let res = ps.read::<i32>(p_vsync);
            if let Ok(v) = res {
                if v != 0 {
                    let res = ps.write::<i32>(p_vsync, &0);
                    if res.is_err() {
                        eprintln!("failed to write VSync");
                    } else {
                        eprintln!("VSync forcibly disabled");
                    }
                }
            }
        }
    }
}

fn scan_fps_ptr(_ps: &Process, m: &Module) -> Result<*mut u8, Box<dyn Error>> {
    let p_fps_anchor = m
        .pattern_scan("7F 0F 8B 05 ? ? ? ?")
        .ok_or("FPS anchor pattern not found")?;
    unsafe {
        let rel = *(p_fps_anchor.add(4) as *mut i32) as isize;
        Ok(p_fps_anchor.offset(rel + 8))
    }
}

fn scan_vsync_ptr(ps: &Process, m: &Module) -> Result<*mut u8, Box<dyn Error>> {
    let p_vsync_anchor = m
        .pattern_scan("E8 ? ? ? ? 8B E8 49 8B 1E")
        .ok_or("VSync anchor pattern not found")?;
    unsafe {
        let rel = *(p_vsync_anchor.add(1) as *mut i32) as isize;
        let p_func_read_vsync = p_vsync_anchor.offset(rel + 5);

        let rel = *(p_func_read_vsync.add(3) as *mut i32) as isize;
        let pp_vsync_base = p_func_read_vsync.offset(rel);

        let vsync_offset = (p_func_read_vsync.add(9) as *mut i32) as isize;

        let p_vsync_base = loop {
            let p = ps.read::<u64>(pp_vsync_base)?;
            if p == 0 {
                sleep(Duration::from_millis(200));
                continue;
            }
            break (p as *mut u8);
        };
        Ok(p_vsync_base.offset(vsync_offset))
    }
}

fn path_exists(path: &str) -> bool {
    unsafe {
        let mut v: Vec<u16> = OsString::from(path).encode_wide().collect();
        v.push(0u16);
        let path = PCWSTR(v.as_ptr());
        let attrs = GetFileAttributesW(path);
        attrs != INVALID_FILE_ATTRIBUTES
    }
}
