extern crate notify;
extern crate xdg;
extern crate regex;
extern crate lazy_static;

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use std::fs::{self, File};
use std::io::{BufReader, BufRead, Result as IoResult, Seek, SeekFrom, Write};

use lazy_static::lazy_static;
use notify::{Watcher, RecursiveMode};
use notify::DebouncedEvent as FsEvent;
use regex::Regex;
use xdg::BaseDirectories;

lazy_static! {
    static ref DSK_NAME: Regex = Regex::new(r"^Name ?= ?(.*)$").unwrap();
}

fn watch_dirs(dirs: &Vec<PathBuf>) -> notify::Result<mpsc::Receiver<FsEvent>> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::watcher(tx, Duration::from_secs(10))?;

    for dir in dirs {
        if let Err(x) = watcher.watch(dir, RecursiveMode::Recursive) {
            eprintln!("Could not watch {:?}: {:?}", dir, x);
        }
    }

    Ok(rx)
}

fn read_desktop_entry(file: PathBuf) -> Option<String> {
    let file_name = file.to_str()?;
    
    let file = File::open(&file).ok()?;
    let lines = BufReader::new(&file).lines();

    let match_line = |line: String| {
        DSK_NAME.captures(&line).map(|x| x[1].to_owned())
    };
    let name = lines
        .filter_map(|l| l.ok())
        .map(match_line)
        .fold(None, |acc, r| acc.or(r));

    let out = format!("{} .%%. {}", name?, file_name);
    Some(out)
}

fn read_desktop_entries(dirs: &Vec<PathBuf>) -> String {
    let to_files = |p: &PathBuf| fs::read_dir(p).ok();
    let string_join = |prev, new: String| prev + new.as_str() + "\n";

    dirs.iter()
        .filter_map(to_files)
        .flatten()
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .filter_map(read_desktop_entry)
        .fold(String::new(), string_join)
}

struct IoContext {
    output_file: File,
    dsk_dirs: Vec<PathBuf>
}

impl IoContext {
    fn find_files() -> Self {
        let xdg_dirs = BaseDirectories::new()
        .expect("Could not use XDG CONFIG cache directory!");
        let desktop_list = xdg_dirs.place_cache_file("xdg-desktop-list")
            .expect("Could not use XDG CONFIG cache directory!");
        
        let mut directories = xdg_dirs.get_data_dirs();
        directories.push(xdg_dirs.get_data_home());

        let join_applications = |x: &mut PathBuf| *x = x.join("applications");
        directories.iter_mut().for_each(join_applications);

        let out_file = File::create(&desktop_list)
            .expect(&format!("Could not create output file {:?}", desktop_list));

        Self {
            output_file: out_file,
            dsk_dirs: directories
        }
    }

    fn write_desktop_list(&mut self) -> IoResult<()> {
        let contents = read_desktop_entries(&self.dsk_dirs);
        self.output_file.seek(SeekFrom::Start(0))?;
        let out_dat = contents.as_bytes();
        self.output_file.write_all(out_dat)?;
        self.output_file.set_len(out_dat.len() as u64)?;
        self.output_file.flush()
    }
}

fn main() {
    let mut io_ctx = IoContext::find_files();

    io_ctx.write_desktop_list()
        .unwrap_or_else(|err| eprintln!("{:?}", err));
    
    let fs_events = watch_dirs(&io_ctx.dsk_dirs)
        .expect("Could not watch desktop file directories!");

    while let Ok(event) = fs_events.recv() {
        match event {
            FsEvent::Error(..) | FsEvent::Chmod(..) => {}
            _ => {
                io_ctx.write_desktop_list()
                    .unwrap_or_else(|err| eprintln!("{:?}", err))
            }
        }
    }
}
