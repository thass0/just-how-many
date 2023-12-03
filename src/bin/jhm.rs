use clap::{Parser, Subcommand};
use url::Url;
use anyhow::Context;
use uuid::Uuid;

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
	/// Page to get the number of hits of.
	url: Url,
    },
    /// Generate the CSS that's needed to track a page.
    Generate {
	/// URL of the page to track.
	url: Url,
    },
}

use Commands::*;

const ADDRESS: &str = "http://localhost:8080"; // TODO: Get from envvar.

fn get_hits(client: &reqwest::blocking::Client, url: &Url) -> anyhow::Result<i32> {
    let response = client.
	get(format!("{ADDRESS}/hits"))
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
	anyhow::bail!("Server error: {err}")
    }
}

fn post_register(client: &reqwest::blocking::Client, url: &Url) -> anyhow::Result<Uuid> {
    let body = format!("url={url}");
    let response = client
        .post(&format!("{ADDRESS}/register"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .context("reqwest POST failed")?;

    if response.status().is_success() {
	response.json::<Uuid>()
	    .context("Failed to decode page ID")
    } else {
	let err = response
	    .text()
	    .context("Failed to receive error message")?;
	anyhow::bail!("Server error: {err}")
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
	Generate { url } => {
	    let page_id = post_register(&client, &url)
		.expect(&format!("Failed to register {url}"));
	    println!(r#"🗃️ SUCCESS! {url} is registered under the page ID {page_id}.
This ID is used to track your page.

Just put the following CSS the in style sheets of the page to track, and you're done!

  body:hover {{
      border-image: url("{ADDRESS}/hit/{page_id}");
      border-width: 0;
  }}"#);
	},
    }
}
