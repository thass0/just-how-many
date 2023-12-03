use std::path::PathBuf;

use clap::{Parser, Subcommand};
use url::Url;
use anyhow::Context;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the number of hits that a page has.
    Hits {
	url: Url,
    },
    /// Register a URL to count page hits on.
    Register {
	url: Url,
    },
}

use Commands::*;

const ADDRESS: &str = "http://localhost:8080"; // TODO: Get from envvar.

fn get_hits(client: &reqwest::blocking::Client, url: &Url) -> anyhow::Result<i32> {
    let response = client.
	get(format!("{}/hits", ADDRESS))
	.query(&[("url", url.as_str())])
	.send()
	.context("reqwest GET failed")?;
    if response.status().is_success() {
	response
	    .json::<i32>()
	    .context("Failed to decode page hit count")
    } else {
	let err = response
	    .text()
	    .context("Failed to receive error message")?;
	Err(anyhow::anyhow!("Server error: {err}"))
    }
}

fn main() {
    let cli = Cli::parse();

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build reqwest client");
    
    match cli.command {
	Hits { url } => {
	    let hits = get_hits(&client, &url)
		.expect(&format!("Failed to get page hits of {url}"));
	    let s = if hits == 1 { "" } else { "s" };
	    println!("🌟 {url} has {hits} hit{s}!");
	},
	Register { url } => println!("Register {url}"),
    }
}
