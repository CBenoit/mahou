use std::collections::HashMap;

use super::{EpisodeNumber, Error, Finder, Result};
use serde::Deserialize;

#[derive(Default)]
pub struct Nibl {
    client: reqwest::blocking::Client,
}

pub const API_BASE: &str = "https://api.nibl.co.uk/nibl";

impl Nibl {
    pub fn search_packages(&self, query: &super::Query) -> Result<Vec<Package>> {
        let mut url = format!(
            "{}/search?query={}%20{}",
            API_BASE, query.search, query.resolution
        );
        if let EpisodeNumber::Number(episode) = query.episode {
            url += &format!("&episodeNumber={}", episode);
        }

        let response = self.client.get(&url).send()?;
        let search_result: SearchResult = response.json()?;
        if search_result.status != "OK" {
            return Err(Error::APIError {
                api: "nibl",
                message: search_result.message,
            });
        }
        Ok(search_result.content)
    }

    fn get_bots(&self) -> Result<HashMap<i64, Bot>> {
        let response = self.client.get(format!("{}/bots", API_BASE)).send()?;
        let result: BotList = response.json()?;
        if result.status != "OK" {
            return Err(Error::APIError {
                api: "nibl",
                message: result.message,
            });
        }
        Ok(result
            .content
            .into_iter()
            .map(|bot| (bot.id, bot))
            .collect())
    }
}

impl Finder for Nibl {
    fn find(&self, query: &super::Query) -> Result<Vec<super::Entry>> {
        let packages = self.search_packages(query)?;
        let bots = self.get_bots()?;

        let latest_episode = packages
            .iter()
            .max_by_key(|p| &p.last_modified)
            .map(|p| p.episode_number)
            .unwrap_or(1);

        let filter_episode = |p: &Package| match query.episode {
            EpisodeNumber::All => true,
            EpisodeNumber::Latest => p.episode_number == latest_episode,
            EpisodeNumber::Number(n) => p.episode_number == n,
        };

        let make_entry = |p: Package| super::Entry {
            package_number: p.number,
            bot_id: p.bot_id,
            bot_name: bots
                .get(&p.bot_id)
                .map(|b| b.name.clone())
                .unwrap_or("unknown bot?".into()),
            name: p.name,
            size: p.size,
        };

        Ok(packages
            .into_iter()
            .filter(filter_episode)
            .map(make_entry)
            .collect())
    }
}

#[derive(Deserialize)]
struct BotList {
    status: String,
    message: String,
    content: Vec<Bot>,
}

#[derive(Deserialize)]
struct Bot {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    bot_id: i64,
    number: i32,
    name: String,
    size: String,
    last_modified: String,
    episode_number: i32,
}

#[derive(Deserialize)]
struct SearchResult {
    status: String,
    message: String,
    content: Vec<Package>,
}
