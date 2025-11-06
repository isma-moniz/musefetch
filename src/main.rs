use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use reqwest::Url;
use rusqlite::{params, Connection, Result as sqlResult, Row, ToSql};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

const USER_AGENT_STR: &str = "musefetch/0.0.1 (hismamoniz@gmail.com)";

// File does not buffer reads and writes.
// For efficiency, consider wrapping the file in a BufReader or BufWriter when performing many small read or write calls,
// unless unbuffered reads and writes are required.

#[derive(Error, Debug)]
pub enum DbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Serialize, Deserialize, Debug)]
struct Artist {
    id: Uuid,
    name: String,
    type_: Option<String>,
    area: Option<String>,
    begin_area: Option<String>,
    disambiguation: Option<String>,
    score: u8,
}

impl Artist {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get::<_, String>("mbid")?.parse().unwrap(),
            name: row.get("name")?,
            type_: row.get("type")?,
            area: row.get("area")?,
            begin_area: row.get("begin_area")?,
            disambiguation: row.get("disambiguation")?,
            score: row.get::<_, i64>("score")? as u8,
        })
    }

    fn insert(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO artists (id, name, type, area, begin_area, disambiguation, score)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                self.id.to_string(),
                self.name,
                self.type_,
                self.area,
                self.begin_area,
                self.disambiguation,
                self.score
            ],
        )?;
        Ok(())
    }
}

// TODO: profile, see if reserve is needed
fn get_db_path() -> std::io::Result<PathBuf> {
    let mut path = dirs::config_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No user config directory found.",
        )
    })?;
    path.push("musefetch");

    std::fs::create_dir_all(&path)?;

    path.push("library.db");
    Ok(path)
}

fn setup_db() -> sqlResult<Connection, DbError> {
    let db_path = get_db_path()?;

    let conn = Connection::open(db_path)?;

    initialize_schema(&conn)?;

    Ok(conn)
}

fn initialize_schema(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        r#"
            CREATE TABLE IF NOT EXISTS artists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT,
                area TEXT,
                begin_area TEXT,
                disambiguation TEXT,
                score INTEGER NOT NULL
            );
        "#,
    )?;
    Ok(())
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

    if text.is_empty() {
        println!("API unavailable right now...");
    } else {
        let v: Value = serde_json::from_str(&text)?;

        if let Some(artist) = process_artist_query(&v, 80) {
            let conn = setup_db()?;
            artist.insert(&conn)?;

            println!("Added artist '{}' to the library!", artist.name);
        } else {
            println!("No artist was selected or matched the threshold...");
        }
    }

    // println!("{:#?}", v);
    Ok(())
}

fn process_artist_query(response: &Value, threshold: u8) -> Option<Artist> {
    let artists = response["artists"].as_array()?;
    println!("Detected {} artists in response!", artists.len());

    let mut artist_vec: Vec<Artist> = artists
        .iter()
        .filter_map(|artist| {
            // parse id
            let id = artist["id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())?;

            // parse score and filter by threshold
            let score = artist["score"].as_u64().map(|s| s as u8)?;
            if score < threshold {
                return None;
            }

            // parse name
            let name = artist["name"].as_str()?.to_owned();

            // parse additional fields
            let type_ = artist["type"].as_str().map(|s| s.to_owned());
            let area = artist["area"]["name"].as_str().map(|s| s.to_owned());
            let begin_area = artist["begin-area"]["name"].as_str().map(|s| s.to_owned());
            let disambiguation = artist["disambiguation"].as_str().map(|s| s.to_owned());

            Some(Artist {
                id,
                name,
                type_,
                area,
                begin_area,
                disambiguation,
                score,
            })
        })
        .collect();

    if artist_vec.is_empty() {
        println!("No artists matched the threshold.");
        return None;
    }

    println!("\nAvailable artists:");
    for (i, artist) in artist_vec.iter().enumerate() {
        println!(
            "[{}] {} (score: {}, type: {:?}, area: {:?})",
            i + 1,
            artist.name,
            artist.score,
            artist.type_,
            artist.area
        );
    }

    print!("\nEnter the number of the artist to love: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let choice = input.trim().parse::<usize>().ok()?;

    if choice == 0 || choice > artist_vec.len() {
        println!("Invalid choice.");
        return None;
    }
    Some(artist_vec.remove(choice - 1))
}

fn handle_love(args: &Vec<String>) {
    if let Some(arg) = args.get(2) {
        if let Err(e) = query_artist(arg) {
            eprintln!("Error while querying artist: {e:?}");
        }
    } else {
        println!("Tell me what to do!");
    }

    // setup_artists();
}

fn main() -> Result<(), DbError> {
    let args: Vec<String> = env::args().collect();
    if let Some(arg) = args.get(1) {
        match arg.as_str() {
            "love" => handle_love(&args),
            _ => println!("Invalid argument"),
        }
    } else {
        println!("Tell me what to do!");
    }
    Ok(())
}
