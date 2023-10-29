use std::{
    env::args,
    io::{stdin, Cursor},
    path::{Path, PathBuf},
    thread,
    time::Duration, str::FromStr,
};
use directories::ProjectDirs;


use anyhow::{anyhow, bail, Result};
use base64::{engine::general_purpose, Engine};
use reqwest::{
    header::{AUTHORIZATION, RANGE},
    Client, RequestBuilder, StatusCode,
};
use rfd::{AsyncFileDialog, FileHandle};
use thirtyfour::{
    components::Component, prelude::ElementQueryable, By, DesiredCapabilities,
    TimeoutConfiguration, WebDriver, WebElement,
};
use tokio::{
    fs::{self, File},
    io::{AsyncWriteExt, BufWriter},
    process::Command,
};

pub const DEFAULT_URL: &str = "https://pubchem.ncbi.nlm.nih.gov/";

pub const LOGIN_USERNAME: &str = "ziemba";
pub const LOGIN_PASSWORD: &str = "chemieisttollundso";

#[tokio::main]
async fn main() -> Result<()> {
    let dirs = ProjectDirs::from("me", "sshcrack", "molecule-orbitals").unwrap();

    env_logger::init();
    let args = args().collect::<Vec<_>>();
    let only_open = args.iter().any(|e| e == "--open-only");

    let app_dir = dirs.data_local_dir();
    let driver_file = app_dir.join("driver.exe");

    let avogadro_loc = app_dir.join("Avogadro");
    if only_open {
        let last = args.last().unwrap();
        let maybe_file = PathBuf::from_str(last).ok().map(|e| e.into_boxed_path());

        run_avogadro(&avogadro_loc, maybe_file.as_deref()).await?;
        return Ok(())
    }


    #[cfg(all(feature = "firefox", feature = "chrome"))]
    panic!("Can not have both features enabled at the same time");

    #[cfg(feature = "chrome")]
    let bin = include_bytes!("./bin/chromedriver.exe");

    #[cfg(feature = "firefox")]
    let bin = include_bytes!("./bin/geckodriver.exe");

    println!("Writing Driver at {:?}", driver_file);
    fs::write(&driver_file, bin).await?;

    println!("Driver wird gestartet...");
    let mut child = Command::new(driver_file).spawn()?;
    let calculated_comp = run_generate().await;

    if let Err(e) = &calculated_comp {
        eprintln!("Fehler: {:?}", e);
    }

    println!("Driver wird beendet...");
    child.start_kill()?;
    child.wait().await?;

    if calculated_comp.is_err() {
        println!("Drücken Sie Enter um das Programm zu beenden...");
        stdin().read_line(&mut String::new())?;

        return Ok(());
    }

    let (_id, file) = calculated_comp.unwrap();
    run_avogadro(&avogadro_loc, Some(&file)).await?;
    Ok(())
}

fn get_avogadro_bin(avogadro_loc: &Path) -> PathBuf {
    avogadro_loc.join("bin/").join("avogadro.exe")
}

pub async fn check_avogadro(avogadro_loc: &Path) -> Result<()> {
    let bin = get_avogadro_bin(avogadro_loc);
    if !bin.is_file() {
        let archive = include_bytes!("./bin/Avogadro.zip");
        println!("Entpacke Avogadro...");
        zip_extract::extract(Cursor::new(archive), &avogadro_loc, true)?;
    }

    Ok(())
}

pub async fn run_avogadro(avogadro_loc: &Path, file: Option<&Path>) -> Result<()> {
    let file = file.and_then(|f| Some(f.to_str().unwrap()));

    check_avogadro(avogadro_loc).await?;

    let bin = get_avogadro_bin(avogadro_loc);
    println!("Starte Avogadro...");
    let mut cmd = Command::new(bin);
    if let Some(f) = file {
        cmd.arg(f);
    }

    cmd.spawn()?;
    Ok(())
}

pub async fn run_generate() -> Result<(String, Box<Path>)> {
    let to_inject = include_str!("./inject.js");
    let error_script = include_str!("./show_error.js");

    println!("Browser wird geöffnet...");

    #[cfg(feature = "chrome")]
    let caps = DesiredCapabilities::chrome();
    #[cfg(feature = "firefox")]
    let caps = DesiredCapabilities::firefox();
    #[cfg(feature = "chrome")]
    let url = "http://localhost:9515";
    #[cfg(feature = "firefox")]
    let url = "http://localhost:4444";

    let driver = WebDriver::new(url, caps).await?;
    let long_duration = Some(Duration::new(99254740991, 0));
    driver
        .update_timeouts(TimeoutConfiguration::new(
            long_duration,
            long_duration,
            None,
        ))
        .await?;

    driver.goto(DEFAULT_URL).await?;
    loop {
        let url = driver.current_url().await?;
        let path = url.path();

        let is_homepage = path.replace("/", "").is_empty();
        let molecule_selected = path.contains("compound");
        let is_search = url.query().is_some_and(|e| e.contains("query="));

        let is_valid = is_search || molecule_selected || is_homepage;

        if !is_valid {
            driver.execute_async(error_script, Vec::new()).await?;
            driver.goto(DEFAULT_URL).await?;
        }

        if !molecule_selected {
            continue;
        }

        driver.query(By::ClassName("grid-flow-col")).first().await?;
        driver.execute_async(to_inject, Vec::new()).await?;

        println!("Waiting for element...");
        let res = driver
            .execute_async(
                r#"
            const done = arguments[0];

            setInterval(() => {
                const element = document.querySelector("\#calculator-molecule-confirmed")
                if(!element)
                    return;

                done()
            }, 350)
        "#,
                Vec::new(),
            )
            .await;

        if let Err(e) = res {
            let e_str = format!("{:?}", e);
            if e_str.contains("Document was unloaded") {
                println!("Document was unloaded. Reloading...");
                continue;
            }

            return Err(e.into());
        }

        break;
    }

    let segments = driver.current_url().await?;
    let segments = segments.path_segments();
    let component_id = segments.unwrap().last().unwrap();

    println!("Berechne Molekül mit ID {}...\nDie Datei wird stückweise ausgegeben und anschließend gespeichert.", component_id);
    driver.close_window().await?;

    let mut done = false;
    let mut total_body = String::new();

    let client = Client::new();
    while !done {
        let mut range = None;
        if !total_body.is_empty() {
            range = Some(format!("bytes={}-", total_body.as_bytes().len() -1));
        }

        let req = get_req(&client, component_id, range).build()?;
        let res = client.execute(req).await?;


        let status = res.status();
        done = res.headers().get("x-processing").is_none();

        let text = res.text().await?;
        println!("{}", text);

        total_body.push_str(&text);
        let res: Result<()> = match status {
            StatusCode::INTERNAL_SERVER_ERROR => {
                println!("Internal Server Error. Stopping. ({})", text);
                bail!("Internal Server Error")
            }
            StatusCode::CREATED => {
                println!("Der Server hat die Berechnung gestartet.");
                Ok(())
            }
            StatusCode::ACCEPTED => Ok(()),
            StatusCode::OK => Ok(()),
            StatusCode::PARTIAL_CONTENT => Ok(()),
            StatusCode::RANGE_NOT_SATISFIABLE => {
                //TODO jank
                total_body.clear();
                Ok(())
            },
            _ => Err(anyhow!("Unbekannter Status Code: {}", status)),
        };

        res?;
        thread::sleep(Duration::from_millis(500))
    }

    let path = download_file(component_id).await?;
    Ok((component_id.to_string(), path))
}

async fn open_dialog() -> Result<Option<FileHandle>> {
    let res = AsyncFileDialog::new()
        .add_filter("Molecule Orbitals", &["out"])
        .set_title("Speicherort auswählen")
        .save_file()
        .await;

    Ok(res)
}

async fn download_file(component_id: &str) -> Result<Box<Path>> {
    let c = Client::new();
    let req = get_req(&c, component_id, None).build()?;

    let mut out_file = None;
    for _ in 0..3 {
        println!("Bitte  Speicherort auswählen...");
        out_file = open_dialog().await?;

        if out_file.is_some() {
            break;
        }
    }

    if out_file.is_none() {
        return Err(anyhow!("Kein Speicherort ausgewählt. Abbruch..."))
    }

    let out_file = out_file.unwrap();
    println!("Lade Datei herunter...");
    let mut res = c.execute(req).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}", res.headers());

    let mut buffer = BufWriter::new(File::create(out_file.path()).await?);

    while let Some(chunk) = res.chunk().await? {
        buffer.write_all(&chunk).await?;
    }

    println!("Datei wurde gespeichert.");

    Ok(out_file.path().to_path_buf().into_boxed_path())
}

fn get_req(c: &Client, component_id: &str, range: Option<String>) -> RequestBuilder {
    let auth_str = format!("{}:{}", LOGIN_USERNAME, LOGIN_PASSWORD);
    let auth_str = general_purpose::STANDARD.encode(auth_str.as_bytes());

    let url = format!("https://chem.sshcrack.me/calculate/{}", component_id);

    let mut res = c
        .get(url)
        .header(AUTHORIZATION, format!("Basic {}", auth_str));

    if let Some(range) = range {
        res = res.header(RANGE, range);
    }

    res
}

/// This component shows how to nest components inside others.
#[derive(Debug, Clone, Component)]
pub struct WrapperComponent {
    base: WebElement, // This is the outer <div>
}
