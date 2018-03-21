extern crate app_dirs;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate failure;
extern crate reqwest;
extern crate webbrowser;

use app_dirs::{app_root, AppDataType, AppInfo};
use chrono::Local;
use clap::SubCommand;
use failure::Error;

use std::fs::File;
use std::io::{Error as ioError, ErrorKind, Read, Write};

const URL: &'static str = "http://plany.wel.wat.edu.pl/lato/E7Q2S1.htm";
const APP_INFO: AppInfo = AppInfo {
    name: crate_name!(),
    author: "Shadlock0133",
};

fn main() {
    let matches = app_from_crate!()
        .subcommand(
            SubCommand::with_name("add").about("Add new website to cache"),
        )
        .subcommand(
            SubCommand::with_name("update")
                .about("Updates cache without opening it"),
        )
        .subcommand(
            SubCommand::with_name("open")
                .about("Open cached file in browser without updating it"),
        )
        .get_matches();

    match matches.subcommand_name() {
        None | Some("update") => update().expect("Couldn't update plan"),
        Some("open") => open().expect("Couldn't open plan"),
        Some(_) => panic!("Subcommand not supported"),
    }
}

// Opening local html files using webbrowser crate
// doesn't work on Windows so we use this workaround
#[cfg(target_os = "windows")]
fn open_file(path: &str) -> ::std::io::Result<()> {
    use std::path::Path;
    use std::process::Command;

    let file_exist = Path::new(path).exists();

    if file_exist {
        Command::new("cmd").arg("/C").arg(path).status().unwrap();

        Ok(())
    } else {
        let msg = format!("File {} doesn't exist", path);
        Err(ioError::new(ErrorKind::NotFound, msg))
    }
}

#[cfg(not(target_os = "windows"))]
fn open_file(path: &str) -> ::std::io::Result<()> {
    webbrowser::open(path).map(|_| ())
}

fn open() -> Result<(), Error> {
    let filename = "index.html";
    let path = app_root(AppDataType::UserData, &APP_INFO)?.join(filename);
    let msg = format!("Couldn't open {:?}", path);
    let path = path.to_str()
        .ok_or(ioError::new(ErrorKind::InvalidInput, msg))?;
    open_file(path)?;
    Ok(())
}

fn update() -> Result<(), Error> {
    let new_file = download_new_file()?;
    let cached_file = get_cached_timestamp()
        .and_then(|timestamp| fetch_cached_file(&timestamp));
    match cached_file {
        Ok(cached_plan) => {
            if &cached_plan == &new_file {
                println!("Files equal, nothing to do.");
                return Ok(());
            } else {
                println!("Files not equal, caching new.");
            }
        }
        Err(_) => {
            println!("No old timestamp, creating new");
        }
    }
    save_new_file(&new_file)?;
    Ok(())
}

fn get_new_filename(timestamp: &str) -> String {
    format!("plan_{}.html", timestamp)
}

fn get_cached_timestamp() -> Result<String, Error> {
    let path = app_root(AppDataType::UserData, &APP_INFO)?.join("timestamp");
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn fetch_cached_file(timestamp: &str) -> Result<Vec<u8>, Error> {
    let filename = get_new_filename(timestamp);
    let path = app_root(AppDataType::UserData, &APP_INFO)?.join(filename);
    let mut file = File::open(path)?;
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn get_current_timestamp() -> Result<String, Error> {
    let dt = Local::now();
    let timestamp = dt.format("%y%m%d%H%M%S").to_string();
    Ok(timestamp)
}

fn download_new_file() -> Result<Vec<u8>, Error> {
    let mut resp = reqwest::get(URL)?;
    let mut buf: Vec<u8> = Vec::new();
    resp.copy_to(&mut buf)?;
    Ok(buf)
}

fn save_new_file(buf: &[u8]) -> Result<(), Error> {
    let timestamp = get_current_timestamp()?;

    // index.html
    let filename = "index.html";
    let path = app_root(AppDataType::UserData, &APP_INFO)?.join(filename);
    let mut file = File::create(path)?;
    file.write_all(buf)?;

    // plan_<timestamp>.html
    let filename = get_new_filename(&timestamp);
    let path = app_root(AppDataType::UserData, &APP_INFO)?.join(filename);
    let print_path = path.to_string_lossy().to_string();
    let mut file = File::create(path)?;
    file.write_all(buf)?;

    // timestamp
    let ts_path = app_root(AppDataType::UserData, &APP_INFO)?.join("timestamp");
    let mut ts_file = File::create(ts_path)?;
    ts_file.write_all(timestamp.as_bytes())?;

    println!("Saved as {}", print_path);
    Ok(())
}
