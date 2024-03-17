use super::database::{Alliance, DataTable, Island, Offset, Player, Town};
use super::offset_data;
use anyhow::Context;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tracing::info;

fn download_generic<U>(
    client: &reqwest::blocking::Client,
    url: U,
) -> std::result::Result<String, reqwest::Error>
where
    U: reqwest::IntoUrl + std::fmt::Display,
{
    let url_text = format!("{url}");
    let result = client.get(url).send()?;
    info!("Got status {} for url {}", result.status(), url_text);
    let text = result.text()?;

    Ok(text)
}

fn make_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent("Rust Grepolis Map - Turun")
        .gzip(true)
        .deflate(true)
        .build()
        .unwrap()
}

impl<'a> DataTable<'a> {
    /// fetches data from the api and saves the processed data to self
    pub fn create_for_world(&'a mut self, server_id: &str) -> anyhow::Result<()> {
        let reqwest_client = make_client();

        let thread_client = reqwest_client.clone();
        let thread_server_id = String::from(server_id);
        let handle_data_players = std::thread::spawn(move || {
            download_generic(
                &thread_client,
                format!("https://{thread_server_id}.grepolis.com/data/players.txt"),
            )
        });
        let thread_client = reqwest_client.clone();
        let thread_server_id = String::from(server_id);
        let handle_data_alliances = std::thread::spawn(move || {
            download_generic(
                &thread_client,
                format!("https://{thread_server_id}.grepolis.com/data/alliances.txt"),
            )
        });
        let thread_client = reqwest_client.clone();
        let thread_server_id = String::from(server_id);
        let handle_data_towns = std::thread::spawn(move || {
            download_generic(
                &thread_client,
                format!("https://{thread_server_id}.grepolis.com/data/towns.txt"),
            )
        });
        let thread_client = reqwest_client;
        let thread_server_id = String::from(server_id);
        let handle_data_islands = std::thread::spawn(move || {
            download_generic(
                &thread_client,
                format!("https://{thread_server_id}.grepolis.com/data/islands.txt"),
            )
        });

        self.offsets = Self::make_offsets();

        let data_alliances = handle_data_alliances
            .join()
            .expect("Failed to join AllianceData fetching thread")
            .context("Failed to download alliance data")?;
        self.alliances = Self::parse_alliances(&data_alliances)?;

        let data_islands = handle_data_islands
            .join()
            .expect("Failed to join islandData fetching thread")
            .context("Failed to download island data")?;
        self.islands = Self::parse_islands(&data_islands)?;

        let data_players = handle_data_players
            .join()
            .expect("Failed to join PlayerData fetching thread")
            .context("Failed to download player data")?;
        self.players = Self::parse_players(&data_players, &self.alliances)?;

        let data_towns = handle_data_towns
            .join()
            .expect("Failed to join TownData fetching thread")
            .context("Failed to download town data")?;
        self.towns = Self::parse_towns(&data_towns, &self.players, &self.islands, &self.offsets)?;

        return Ok(());
    }

    fn make_offsets() -> HashMap<u8, Offset> {
        let lines: Vec<&str> = offset_data::OFFSET_DATA.lines().collect();
        let mut re = HashMap::with_capacity(lines.len());
        for line in lines {
            let mut values = line.split(',');
            let typ: u8 = values.next().unwrap().parse().unwrap();
            let x: u16 = values.next().unwrap().parse().unwrap();
            let y: u16 = values.next().unwrap().parse().unwrap();
            let slot_number: u8 = values.next().unwrap().parse().unwrap();
            re.insert(
                slot_number,
                Offset {
                    typ,
                    x,
                    y,
                    slot_number,
                },
            );
        }
        return re;
    }

    fn parse_alliances(data: &str) -> anyhow::Result<HashMap<u32, Alliance>> {
        let lines: Vec<&str> = data.lines().collect();
        let mut re = HashMap::with_capacity(lines.len());
        for line in lines {
            let mut values = line.split(',');

            let id = values
                .next()
                .with_context(|| format!("No ally id in {line}"))?
                .parse()
                .with_context(|| format!("No ally id in {line} that can be parsed as int"))?;
            let name = {
                let text = values
                    .next()
                    .with_context(|| format!("No ally name in {line}"))?;
                let decoded = form_urlencoded::parse(text.as_bytes())
                    .map(|(key, val)| [key, val].concat())
                    .collect::<String>();
                decoded
            };
            let points = values
                .next()
                .with_context(|| format!("No ally pts in {line}"))?
                .parse()
                .with_context(|| format!("No ally points in {line} that can be parsed as int"))?;
            let towns = values
                .next()
                .with_context(|| format!("No ally towns in {line}"))?
                .parse()
                .with_context(|| format!("No ally towns in {line} that can be parsed as int"))?;
            let members = values
                .next()
                .with_context(|| format!("No ally membrs in {line}"))?
                .parse()
                .with_context(|| format!("No ally members in {line} that can be parsed as int"))?;
            let rank = values
                .next()
                .with_context(|| format!("No ally rank in {line}"))?
                .parse()
                .with_context(|| format!("No ally rank in {line} that can be parsed as int"))?;
            re.insert(
                id,
                Alliance {
                    id,
                    name,
                    points,
                    towns,
                    members,
                    rank,
                },
            );
        }
        return Ok(re);
    }

    fn parse_islands(data: &str) -> anyhow::Result<HashMap<(u16, u16), Island>> {
        let lines: Vec<&str> = data.lines().collect();
        let mut re = HashMap::with_capacity(lines.len());
        for line in lines {
            let mut values = line.split(',');

            let id = values
                .next()
                .with_context(|| format!("No island id in {line}"))?
                .parse()
                .with_context(|| format!("No island id in {line} that can be parsed as int"))?;
            let x = values
                .next()
                .with_context(|| format!("No island x in {line}"))?
                .parse()
                .with_context(|| format!("No island x in {line} that can be parsed as int"))?;
            let y = values
                .next()
                .with_context(|| format!("No island y in {line}"))?
                .parse()
                .with_context(|| format!("No island y in {line} that can be parsed as int"))?;
            let typ = values
                .next()
                .with_context(|| format!("No island type in {line}"))?
                .parse()
                .with_context(|| format!("No island type in {line} that can be parsed as int"))?;
            let towns = values
                .next()
                .with_context(|| format!("No island towns in {line}"))?
                .parse()
                .with_context(|| format!("No island towns in {line} that can be parsed as int"))?;
            let ressource_plus = values
                .next()
                .with_context(|| format!("No island res+ in {line}"))?
                .to_string();
            let ressource_minus = values
                .next()
                .with_context(|| format!("No island res- in {line}"))?
                .to_string();
            re.insert(
                (x, y),
                Island {
                    id,
                    x,
                    y,
                    typ,
                    towns,
                    ressource_plus,
                    ressource_minus,
                },
            );
        }
        return Ok(re);
    }

    fn parse_players<'b>(
        data: &str,
        alliances: &'b HashMap<u32, Alliance>,
    ) -> anyhow::Result<HashMap<u32, Player<'b>>> {
        let lines: Vec<&str> = data.lines().collect();
        let mut re = HashMap::with_capacity(lines.len());
        for line in lines {
            let mut values = line.split(',');

            let id = values
                .next()
                .with_context(|| format!("No player id in {line}"))?
                .parse()
                .with_context(|| format!("No player id in {line} that can be parsed as int"))?;
            let name = {
                let text = values
                    .next()
                    .with_context(|| format!("No player name in {line}"))?;
                let decoded = form_urlencoded::parse(text.as_bytes())
                    .map(|(key, val)| [key, val].concat())
                    .collect::<String>();
                decoded
            };
            let opt_alliance_id = {
                let text = values
                    .next()
                    .with_context(|| format!("No player alliance id in {line}"))?;
                if text.is_empty() {
                    None
                } else {
                    Some(text.parse().with_context(|| {
                        format!("No player alliance id in {line} that can be parsed as int")
                    })?)
                }
            };
            let points = values
                .next()
                .with_context(|| format!("No player points in {line}"))?
                .parse()
                .with_context(|| format!("No player point in {line} that can be parsed as int"))?;
            let rank = values
                .next()
                .with_context(|| format!("No player rank in {line}"))?
                .parse()
                .with_context(|| format!("No player rank in {line} that can be parsed as int"))?;
            let towns = values
                .next()
                .with_context(|| format!("No player towns in {line}"))?
                .parse()
                .with_context(|| format!("No player towns in {line} that can be parsed as int"))?;

            let opt_alliance_tuple = if let Some(alliance_id) = opt_alliance_id {
                let opt_alliance = alliances.get(&alliance_id);
                if let Some(alliance) = opt_alliance {
                    Some((alliance_id, alliance))
                } else {
                    None
                }
            } else {
                None
            };

            re.insert(
                id,
                Player {
                    id,
                    name,
                    alliance: opt_alliance_tuple,
                    points,
                    rank,
                    towns,
                },
            );
        }
        return Ok(re);
    }

    #[allow(clippy::cast_lossless)]
    fn parse_towns<'c>(
        data: &str,
        players: &'c HashMap<u32, Player>,
        islands: &'c HashMap<(u16, u16), Island>,
        offsets: &'c HashMap<u8, Offset>,
    ) -> anyhow::Result<HashMap<u32, Town<'c>>> {
        let lines: Vec<&str> = data.lines().collect();
        let mut re = HashMap::with_capacity(lines.len());
        for line in lines {
            let mut values = line.split(',');
            let id = values
                .next()
                .with_context(|| format!("No town id in {line}"))?
                .parse()
                .with_context(|| format!("No town id in {line} that can be parsed as int"))?;
            let opt_player_id: Option<u32> = {
                let text = values
                    .next()
                    .with_context(|| format!("No town player id in {line}"))?;
                if text.is_empty() {
                    None
                } else {
                    Some(text.parse().with_context(|| {
                        format!("No town player id in {line} that can be parsed as int")
                    })?)
                }
            };
            let name = {
                let text = values
                    .next()
                    .with_context(|| format!("No town name in {line}"))?;
                let decoded = form_urlencoded::parse(text.as_bytes())
                    .map(|(key, val)| [key, val].concat())
                    .collect::<String>();
                decoded
            };
            let x = values
                .next()
                .with_context(|| format!("No town x in {line}"))?
                .parse()
                .with_context(|| format!("No town x in {line} that can be parsed as int"))?;
            let y = values
                .next()
                .with_context(|| format!("No town y in {line}"))?
                .parse()
                .with_context(|| format!("No town y in {line} that can be parsed as int"))?;
            let slot_number = values
                .next()
                .with_context(|| format!("No town slot_number in {line}"))?
                .parse()
                .with_context(|| {
                    format!("No town slot_number in {line} that can be parsed as int")
                })?;
            let points = values
                .next()
                .with_context(|| format!("No town points in {line}"))?
                .parse()
                .with_context(|| format!("No town points in {line} that can be parsed as int"))?;

            // get actual player from the player id
            let opt_player_tuple = if let Some(player_id) = opt_player_id {
                let opt_player = players.get(&player_id);
                if let Some(alliance) = opt_player {
                    Some((player_id, alliance))
                } else {
                    None
                }
            } else {
                None
            };

            // get actual island from x and y
            let island_tuple = (x, y, islands.get(&(x, y)).unwrap());

            // get the offset from the offset list from slot_number
            let offset_tuple = (slot_number, offsets.get(&slot_number).unwrap());

            // compute actual x
            let actual_x = x as f32 + offset_tuple.1.x as f32 / 125f32;
            let actual_y = y as f32 + offset_tuple.1.y as f32 / 125f32;

            re.insert(
                id,
                Town {
                    id,
                    name,
                    points,
                    player: opt_player_tuple,
                    island: island_tuple,
                    offset: offset_tuple,
                    actual_x,
                    actual_y,
                },
            );
        }
        return Ok(re);
    }
}
