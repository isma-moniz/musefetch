use std::env;
use std::fs::File;
use std::io::prelude::*;

extern crate dirs;
extern crate serde_json;
extern crate serde;
extern crate uuid;

use serde::{Deserialize, Serialize};
use serde_json::Result;
use uuid::Uuid;

// File does not buffer reads and writes. 
// For efficiency, consider wrapping the file in a BufReader or BufWriter when performing many small read or write calls, 
// unless unbuffered reads and writes are required.

#[derive(Serialize, Deserialize)]
struct Artist {
    name: String,
    mb_id: Uuid,
    albums: Vec<String>
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(arg) = args.get(1) {
        match arg.as_str() {
            "love" => handle_love(&args),
            _ => println!("Invalid argument")
        }
    } else {
        println!("Tell me what to do!");
    }
}

fn setup_artists() -> std::io::Result<()> {
    match dirs::config_dir() {
        Some(mut path) => {
            path.reserve("/musefetch/loved_artists.txt".len());
            path.push("musefetch");
            std::fs::create_dir_all(&path)?;
            path.push("loved_artists.txt");
            let artists_file = File::create(path)?;
            Ok(())
        }
        None => panic!("Unable to set up home_dir!")
    } 
}

fn handle_love(args: &Vec<String>) {
    setup_artists();
}
