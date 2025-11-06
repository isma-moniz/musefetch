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
use serde_json::Value;
use uuid::Builder;
use uuid::Uuid;

const USER_AGENT_STR: &str = "musefetch/0.0.1 (hismamoniz@gmail.com)";

// File does not buffer reads and writes.
// For efficiency, consider wrapping the file in a BufReader or BufWriter when performing many small read or write calls,
// unless unbuffered reads and writes are required.

#[derive(Serialize, Deserialize, Debug)]
struct Artist {
    id: Uuid,
    type_: String,
    score: u8,
    name: String,
    area: String,
    disambiguation: Option<String>,
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

    let v: Value = serde_json::from_str(&text)?;

    process_artist_query(&v);

    // println!("{:#?}", v);
    Ok(())
}

fn process_artist_query(response: &Value) {
    if response["artists"].is_array() {
        println!("Detected artists array in response!")
    } else {
        panic!("No artists array in response.")
    }

    let artists = response["artists"].as_array().unwrap();
    for artist in artists {
        let area: String;
        if !artist["begin-area"].is_null() {
            area = artist["begin-area"]["name"].as_str().unwrap().to_owned();
        } else if !artist["area"].is_null() {
            area = artist["area"]["name"].as_str().unwrap().to_owned();
        } else {
            area = "Unknown origin".to_owned();
        }

        let artist_id_str = artist["id"].as_str().unwrap();
        let id = Uuid::parse_str(artist_id_str);
        if id.is_err() {
            panic!("Error parsing artist id!");
        }
        let id = id.unwrap();
        let name = artist["name"].as_str().unwrap().to_owned();
        let score: u8 = artist["score"].as_u64().unwrap() as u8;

        let type_: String;
        if !artist["type"].is_null() {
            type_ = artist["type"].as_str().unwrap().to_owned();
        } else {
            type_ = "Unknown type".to_owned();
        }
        let disambiguation: Option<String>;
        if !artist["disambiguation"].is_null() {
            disambiguation = Some(artist["disambiguation"].as_str().unwrap().to_owned());
        } else {
            disambiguation = None;
        }

        let artist_struct = Artist {
            id,
            type_,
            score,
            area,
            name,
            disambiguation,
        };

        println!("{:#?}", artist_struct);
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
