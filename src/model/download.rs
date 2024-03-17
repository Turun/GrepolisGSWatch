use super::database::{Alliance, DataTable, Island, Offset, Player, Town};
use super::offset_data;
use anyhow::Context;

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

impl DataTable {
    pub fn create_for_world(server_id: &str) -> anyhow::Result<Self> {
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

        let offsets = Self::make_offsets();

        let data_alliances = handle_data_alliances
            .join()
            .expect("Failed to join AllianceData fetching thread")
            .context("Failed to download alliance data")?;
        let alliances = Self::parse_alliances(&data_alliances)?;

        let data_islands = handle_data_islands
            .join()
            .expect("Failed to join islandData fetching thread")
            .context("Failed to download island data")?;
        let islands = Self::parse_islands(&data_islands)?;

        let data_players = handle_data_players
            .join()
            .expect("Failed to join PlayerData fetching thread")
            .context("Failed to download player data")?;
        let players = Self::parse_players(&data_players, &alliances)?;

        let data_towns = handle_data_towns
            .join()
            .expect("Failed to join TownData fetching thread")
            .context("Failed to download town data")?;
        let towns = Self::parse_towns(&data_towns, &players, &islands, &offsets)?;

        Ok(Self {
            offsets,
            islands,
            alliances,
            players,
            towns,
        })
    }

    fn make_offsets() -> Vec<Arc<Offset>> {
        let lines: Vec<&str> = offset_data::OFFSET_DATA.lines().collect();
        let mut re = Vec::with_capacity(lines.len());
        for line in lines {
            let mut values = line.split(',');
            let typ: u8 = values.next().unwrap().parse().unwrap();
            let x: u16 = values.next().unwrap().parse().unwrap();
            let y: u16 = values.next().unwrap().parse().unwrap();
            let slot_number: u8 = values.next().unwrap().parse().unwrap();
            re.push(Arc::new(Offset {
                typ,
                x,
                y,
                slot_number,
            }));
        }
        return re;
    }

    fn parse_alliances(data: &str) -> anyhow::Result<Vec<Arc<Alliance>>> {
        let lines: Vec<&str> = data.lines().collect();
        let mut re = Vec::with_capacity(lines.len());
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
            re.push(Arc::new(Alliance {
                id,
                name,
                points,
                towns,
                members,
                rank,
            }));
        }
        return Ok(re);
    }

    fn parse_islands(data: &str) -> anyhow::Result<Vec<Arc<Island>>> {
        let lines: Vec<&str> = data.lines().collect();
        let mut re = Vec::with_capacity(lines.len());
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
            re.push(Arc::new(Island {
                id,
                x,
                y,
                typ,
                towns,
                ressource_plus,
                ressource_minus,
            }));
        }
        return Ok(re);
    }

    fn parse_players(data: &str, alliances: &[Arc<Alliance>]) -> anyhow::Result<Vec<Arc<Player>>> {
        let lines: Vec<&str> = data.lines().collect();
        let mut re = Vec::with_capacity(lines.len());
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

            let alliance_tuple = if let Some(alliance_id) = opt_alliance_id {
                let opt_alliance = alliances.iter().find(|a| a.id == alliance_id);
                opt_alliance.map(|alliance| (alliance_id, Arc::clone(alliance)))
            } else {
                None
            };

            re.push(Arc::new(Player {
                id,
                name,
                alliance: alliance_tuple,
                points,
                rank,
                towns,
            }));
        }
        return Ok(re);
    }

    #[allow(clippy::cast_lossless)]
    fn parse_towns(
        data: &str,
        players: &[Arc<Player>],
        islands: &[Arc<Island>],
        offsets: &[Arc<Offset>],
    ) -> anyhow::Result<Vec<Arc<Town>>> {
        let lines: Vec<&str> = data.lines().collect();
        let mut re = Vec::with_capacity(lines.len());
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
            let player_tuple = if let Some(player_id) = opt_player_id {
                let opt_player = players.iter().find(|p| p.id == player_id);
                opt_player.map(|player| (player_id, Arc::clone(player)))
            } else {
                None
            };

            // get actual island from x and y
            let island_tuple = (
                x,
                y,
                Arc::clone(islands.iter().find(|i| i.x == x && i.y == y).unwrap()),
            );
            //     let opt_island = islands.iter().find(|i| i.x == x && i.y == y);
            //     if let Some(island) = opt_island {
            //         Arc::clone(island)
            //     } else {
            //         Arc::clone(&islands[0])
            //     }
            // });

            // get the offset from the offset list from slot_number
            let offset_tuple = (
                slot_number,
                Arc::clone(
                    offsets
                        .iter()
                        .find(|o| o.slot_number == slot_number)
                        .unwrap(),
                ),
            );
            // let opt_offset = offsets.iter().find(|o| o.slot_number == slot_number);
            // if let Some(offset) = opt_offset {
            //     Arc::clone(offset)
            // } else {
            //     Arc::clone(&offsets[0])
            // }
            // });

            // compute actual x
            let actual_x = x as f32 + offset_tuple.1.x as f32 / 125f32;
            let actual_y = y as f32 + offset_tuple.1.y as f32 / 125f32;

            re.push(Arc::new(Town {
                id,
                name,
                points,
                player: player_tuple,
                island: island_tuple,
                offset: offset_tuple,
                actual_x,
                actual_y,
            }));
        }
        return Ok(re);
    }
}
