use std::{fs::File, io::Read};

use anyhow::{bail, Result};
use base64::Engine;
use clap::{Parser, Subcommand};
use regex::Regex;
use rm_config::Config;
use transmission_rpc::types::TorrentAddArgs;

use crate::transmission;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    AddTorrent { torrent: String },
    FetchRss { url: String, filter: Option<String> },
}

pub async fn handle_command(config: &Config, command: Commands) -> Result<()> {
    match command {
        Commands::AddTorrent { torrent } => add_torrent(config, torrent).await?,
        Commands::FetchRss { url, filter } => fetch_rss(config, &url, filter.as_deref()).await?,
    }
    Ok(())
}

async fn add_torrent(config: &Config, torrent: String) -> Result<()> {
    let mut transclient = transmission::utils::client_from_config(&config);
    let args = {
        if torrent.starts_with("magnet:")
            || torrent.starts_with("http:")
            || torrent.starts_with("https:")
        {
            TorrentAddArgs {
                filename: Some(torrent),
                ..Default::default()
            }
        } else if torrent.starts_with("www") {
            TorrentAddArgs {
                filename: Some(format!("https://{torrent}")),
                ..Default::default()
            }
        } else {
            let mut torrent_file = File::open(torrent)?;
            let mut buf = vec![];
            torrent_file.read_to_end(&mut buf).unwrap();
            let metainfo = base64::engine::general_purpose::STANDARD.encode(buf);
            TorrentAddArgs {
                metainfo: Some(metainfo),
                ..Default::default()
            }
        }
    };

    if let Err(e) = transclient.torrent_add(args).await {
        eprintln!("error while adding a torrent: {e}");
        if e.to_string().contains("expected value at line") {
            eprintln!("Check whether your arguments are valid.");
        }

        std::process::exit(1);
    };
    Ok(())
}

async fn fetch_rss(config: &Config, url: &str, filter: Option<&str>) -> Result<()> {
    let mut transclient = transmission::utils::client_from_config(&config);
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = rss::Channel::read_from(&content[..])?;
    let re: Option<Regex> = {
        if let Some(filter_str) = filter {
            let res = Regex::new(filter_str)?;
            Some(res)
        } else {
            None
        }
    };
    let items = channel.items().iter().filter_map(|item| {
        if let (Some(title), Some(url)) = (item.title(), item.link()) {
            if let Some(re) = &re {
                if re.is_match(title) {
                    return Some((title, url));
                }
            } else {
                return Some((title, url));
            }
        }
        None
    });
    for (title, url) in items {
        println!("downloading {title}");
        let args = TorrentAddArgs {
            filename: Some(url.to_string()),
            ..Default::default()
        };
        if let Err(e) = transclient.torrent_add(args).await {
            bail!("error while adding a torrent: {e}")
        }
    }
    Ok(())
}
