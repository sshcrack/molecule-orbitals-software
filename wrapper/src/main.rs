use std::{fs::write, process::Command};

use directories::{ProjectDirs, UserDirs};
use mslnk::ShellLink;

fn main() {
    let user_dir = UserDirs::new().unwrap();

    let bin = include_bytes!("../../main/target/release/main.exe");

    let avo_ico = include_bytes!("./Avogadro.ico");
    let g_ico = include_bytes!("./generate.ico");

    let dirs = ProjectDirs::from("me", "sshcrack", "molecule-orbitals").unwrap();
    let d = dirs.data_local_dir();
    let out_file = d.join("program.exe");

    if !d.exists() {
        std::fs::create_dir_all(&d).unwrap();
    }

    let avogadro_ico = d.join("Avogadro.ico");
    let generate_ico = d.join("generate.ico");

    write(&out_file, bin).unwrap();
    write(&avogadro_ico, avo_ico).unwrap();
    write(&generate_ico, g_ico).unwrap();

    let desktop = user_dir.desktop_dir().unwrap();
    let calculate_lnk = desktop.join("Molekül-Orbitale berechnen.lnk");
    let open_lnk = desktop.join("Molekül-Orbitale öffnen.lnk");

    let mut sl = ShellLink::new(&out_file).unwrap();
    sl.set_icon_location(Some(generate_ico.to_str().unwrap().to_string()));

    sl.create_lnk(&calculate_lnk).unwrap();

    let mut sl = ShellLink::new(&out_file).unwrap();
    sl.set_icon_location(Some(avogadro_ico.to_str().unwrap().to_string()));
    sl.set_arguments(Some("--open-only".to_string()));

    sl.create_lnk(&open_lnk).unwrap();

    Command::new(out_file).spawn().unwrap();
}
