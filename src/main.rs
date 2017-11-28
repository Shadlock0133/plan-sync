extern crate app_dirs;
extern crate reqwest;
extern crate chrono;

use app_dirs::*;
use chrono::prelude::*;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

const URL: &'static str = "http://plany.wel.wat.edu.pl/lato/E7Q2S1.htm";
const APP_INFO: AppInfo = AppInfo { name: "plan-sync", author: "Shadlock0133" }; 

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Box<Error>> {
    let new_plan = get_new_plan().expect("Network error");
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
    save_new_plan(&new_plan)?;
    Ok(())
}

fn get_cached_timestamp() -> Result<String, Box<Error>> {
    let path = app_root(AppDataType::UserData, &APP_INFO)?.join("timestamp");
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

fn get_cached_plan(timestamp: &str) -> Result<Vec<u8>, Box<Error>> {
    let filename = format!("plan_{}.html", timestamp);
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

    let filename = format!("plan_{}.html", timestamp);
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
