use clap::{Parser, Subcommand};
use url::Url;
use anyhow::Context;
use uuid::Uuid;

use jhm::routes::Hits as JhmHits;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(env="JHM_SERVICE")]
    /// Address of the JHM service that does the tracking.
    service: Url,
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

fn get_hits(
    client: &reqwest::blocking::Client,
    service: &Url,
    url: &Url,
) -> anyhow::Result<JhmHits> {
    let mut service = service.clone();
    service.set_path("hits");
    let response = client.
	get(service)
	.query(&[("url", url.as_str())])
	.send()
	.context("reqwest GET failed")?;

    if response.status().is_success() {
	response
	    .json::<JhmHits>()
	    .context("Failed to decode page hit count")
    } else {
	let err = response
	    .text()
	    .context("Failed to receive error message")?;
	anyhow::bail!("Server error: {err}")
    }
}

fn post_register(
    client: &reqwest::blocking::Client,
    service: &Url,
    url: &Url,
) -> anyhow::Result<Uuid> {
    let mut service = service.clone();
    service.set_path("register");

    let body = format!("url={url}");

    let response = client
        .post(service)
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
	    let hits = get_hits(&client, &cli.service, &url)
		.expect(&format!("Failed to get page hits of {url}"));
	    let s = if hits.n == 1 { "" } else { "s" };
	    println!("🌟 {url} has {} hit{s}!", hits.n);
	},
	Generate { url } => {
	    let page_id = post_register(&client, &cli.service, &url)
		.expect(&format!("Failed to register {url}"));
	    println!(r#"🗃️ SUCCESS! {url} is registered under the page ID {page_id}.
This ID is used to track your page.

Just put the following CSS the in style sheets of the page to track, and you're done!

  body:hover {{
      border-image: url("{}/hit/{page_id}");
      border-width: 0;
  }}"#, &cli.service);
	},
    }
}
