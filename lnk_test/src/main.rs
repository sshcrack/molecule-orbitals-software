use std::{env::current_exe, fs::write};

use directories::{ProjectDirs, UserDirs};
use mslnk::ShellLink;
use anyhow::Result;
use same_file::is_same_file;

fn main() -> Result<()> {
    let user_dir = UserDirs::new().unwrap();

    let avo_ico = include_bytes!("./ico/Avogadro.ico");
    let g_ico = include_bytes!("./ico/generate.ico");

    let dirs = ProjectDirs::from("me", "sshcrack", "molecule-orbitals").unwrap();
    let d = dirs.data_local_dir();
    let out_file = d.join("program.exe");

    let curr_exe = current_exe().unwrap();

    if out_file.is_file() && is_same_file(&curr_exe, &out_file)? {
        return Ok(())
    }

    if !d.exists() {
        std::fs::create_dir_all(&d)?;
    }

    let desktop = user_dir.desktop_dir().unwrap();
    let calculate_lnk = desktop.join("Molekül-Orbitale berechnen.lnk");
    let open_lnk = desktop.join("Molekül-Orbitale öffnen.lnk");

    let avogadro_ico = d.join("Avogadro.ico");
    let generate_ico = d.join("generate.ico");

    write(&avogadro_ico, avo_ico)?;
    write(&generate_ico, g_ico)?;
    write(&out_file, vec![])?;


    let mut sl = ShellLink::new(&out_file)?;
    sl.set_icon_location(Some(generate_ico.to_str().unwrap().to_string()));

    sl.create_lnk(&calculate_lnk)?;

    let mut sl = ShellLink::new(&out_file)?;
    sl.set_icon_location(Some(avogadro_ico.to_str().unwrap().to_string()));
    sl.set_arguments(Some("--open-only".to_string()));

    sl.create_lnk(&open_lnk)?;

    Ok(())
}
