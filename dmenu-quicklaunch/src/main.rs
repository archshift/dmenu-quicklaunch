extern crate xdg;

use std::process::{Command, Stdio};
use std::fs;
use std::env;
use std::io::Write;
use std::collections::BTreeMap as Map;

use xdg::BaseDirectories;

fn get_entries() -> Map<String, String> {
    let xdg_dirs = BaseDirectories::new()
        .expect("Could not use XDG CONFIG cache directory!");
    let desktop_list = xdg_dirs.place_cache_file("xdg-desktop-list")
        .expect("Could not use XDG CONFIG cache directory!");

    let in_dat = fs::read_to_string(&desktop_list)
        .expect(&format!("Could not open desktop file list {:?}", desktop_list));
    
    let to_items = |line: &str| { 
        let mut items_it = line.split(" .%%. ");
        Some((
            items_it.next()?.to_owned(),
            items_it.next()?.to_owned()
        ))
    };

    in_dat.split('\n')
        .filter_map(to_items)
        .collect()
}

fn dmenu_choose(all_entries: &Map<String, String>) -> Option<&String> {
    let dmenu_cmd = env::var("DMENU")
        .unwrap_or("dmenu".into());
    let mut dmenu_it = dmenu_cmd.split(' ');
    let dmenu_pgrm = dmenu_it.next().unwrap();

    let mut dmenu = Command::new(dmenu_pgrm)
        .args(dmenu_it)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn dmenu");
    {
        let dmenu_dat = dmenu.stdin.as_mut()
            .expect("Failed to open dmenu stdin");

        for (name, _) in all_entries {
            dmenu_dat.write_all(format!("{}\n", name).as_bytes())
                .expect("Failed to write to dmenu stdin");
        }
    }

    let dmenu_out = dmenu.wait_with_output()
        .expect("Failed to read dmenu output");
    let chosen = String::from_utf8(dmenu_out.stdout)
        .expect("dmenu output not valid UTF-8!");
    let chosen = chosen.trim_end_matches('\n');
    if chosen.is_empty() {
        None
    } else {
        let chosen_dsk = all_entries.get(chosen)
            .expect(&format!("Could not find desktop entry for `{}`", chosen));
        Some(chosen_dsk)
    }
}

fn launch_app(desktop_file: &str) {
    let dex_res = Command::new("dex")
        .arg(desktop_file)
        .status()
        .expect("Failed to spawn dex");
    
    if !dex_res.success() {
        eprintln!("Failed to spawn {}", desktop_file);
    }
}

fn main() {
    let all_entries = get_entries();
    
    if let Some(chosen) = dmenu_choose(&all_entries) {
        launch_app(chosen);
    }
}
