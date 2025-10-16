use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;

extern crate dirs;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const USER_AGENT_STR: &str = "musefetch/0.0.1 (hismamoniz@gmail.com)";

// File does not buffer reads and writes.
// For efficiency, consider wrapping the file in a BufReader or BufWriter when performing many small read or write calls,
// unless unbuffered reads and writes are required.

#[derive(Serialize, Deserialize)]
struct Artist {
    id: Uuid,
    type_: String,
    type_id: Uuid,
    score: u8,
    name: String,
    sort_name: String,
    country: String,
    area: String,
    albums: Vec<String>,
}

#[derive(Deserialize)]
struct ArtistSearch {
    created: String,
    count: u16,
    offset: u16,
    artists: Vec<Artist>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(arg) = args.get(1) {
        match arg.as_str() {
            "love" => handle_love(&args),
            _ => println!("Invalid argument"),
        }
    } else {
        println!("Tell me what to do!");
    }
}

fn query_artist(artist_name: &String) -> Result<()> {
    let url = Url::parse_with_params(
        "https://musicbrainz.org/ws/2/artist",
        &[("query", artist_name.as_str()), ("fmt", "json")],
    )?;

    let response = Client::new()
        .get(url)
        .header(USER_AGENT, USER_AGENT_STR)
        .send()?;
    let text = response.text()?;

    let out = serde_json::from_str(&text)?;

    println!("{:#?}", out);
    Ok(())
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
        None => panic!("Unable to set up home_dir!"),
    }
}

fn handle_love(args: &Vec<String>) {
    if let Some(arg) = args.get(2) {
        let _ = query_artist(arg);
    } else {
        println!("Tell me what to do!");
    }

    // setup_artists();
}
