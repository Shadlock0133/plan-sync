extern crate app_dirs;
extern crate reqwest;
extern crate chrono;
// extern crate clap;
// extern crate failure;

use app_dirs::*;
use chrono::prelude::*;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;

const URL: &'static str = "http://plany.wel.wat.edu.pl/lato/E7Q2S1.htm";
const APP_INFO: AppInfo = AppInfo { name: "plan-sync", author: "Shadlock0133" }; 
const BROWSER: &'static str = "C:\\Program Files\\Mozilla Firefox\\firefox.exe";

enum AppError {
    NetworkError,
    FileWriteError,
}

fn main() {
    match update() {
        Ok(_) | Err(AppError::NetworkError) => open(),
        _ => println!("FIX ME"),
    }
}

fn open() {
    let path = app_root(AppDataType::UserData, &APP_INFO).unwrap()
        .join(get_plan_filename(&get_cached_timestamp().unwrap()));
    Command::new(BROWSER)
        .arg(path)
        .spawn()
        .expect("open() failed");
}

fn update() -> Result<(), AppError> {
    let new_plan = get_new_plan().map_err(|_| AppError::NetworkError)?;
    let cached_plan = get_cached_timestamp()
        .and_then(|timestamp| {
            get_cached_plan(&timestamp)
    });
    match cached_plan {
        Ok(cached_plan) => {
            if &cached_plan == &new_plan {
                println!("Files equal, nothing to do.");
                return Ok(());
            } else {
                println!("Files not equal, caching new.");
            }
        },
        Err(_) => {
            println!("No old timestamp, creating new");
        }
    }
    save_new_plan(&new_plan).map_err(|_| AppError::FileWriteError)?;
    Ok(())
}

fn get_cached_timestamp() -> Result<String, Box<Error>> {
    let path = app_root(AppDataType::UserData, &APP_INFO)?.join("timestamp");
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn get_plan_filename(timestamp: &str) -> String {
    format!("plan_{}.html", timestamp)
}

fn get_cached_plan(timestamp: &str) -> Result<Vec<u8>, Box<Error>> {
    let filename = get_plan_filename(timestamp);
    let path = app_root(AppDataType::UserData, &APP_INFO)?.join(filename);
    let mut file = File::open(path)?;
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn get_current_timestamp() -> Result<String, Box<Error>> {
    let dt = Local::now();
    let timestamp = dt.format("%y%m%d%H%M%S").to_string();
    Ok(timestamp)
}

fn get_new_plan() -> Result<Vec<u8>, Box<Error>> {
    let mut resp = reqwest::get(URL)?;
    let mut buf: Vec<u8> = Vec::new();
    resp.copy_to(&mut buf)?;
    Ok(buf)
}

fn save_new_plan(buf: &[u8]) -> Result<(), Box<Error>> {
    let timestamp = get_current_timestamp()?;

    let filename = get_plan_filename(&timestamp);
    let path = app_root(AppDataType::UserData, &APP_INFO)?.join(filename);
    let print_path = path.to_string_lossy().to_string();
    let mut file = File::create(path)?;
    file.write_all(buf)?;
    
    let ts_path = app_root(AppDataType::UserData, &APP_INFO)?.join("timestamp");
    let mut ts_file = File::create(ts_path)?;
    ts_file.write_all(timestamp.as_bytes())?;

    println!("Saved to {}", print_path);
    Ok(())
}
