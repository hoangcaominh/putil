use postgres::{Client, Error, NoTls};
use uuid::Uuid;

use crate::model::Player;

pub fn get_client() -> Result<Client, Error> {
    let host = std::env::var("DB_HOST").expect("missing database host");
    let port = std::env::var("DB_PORT").unwrap_or(String::from("5432"));
    let user = std::env::var("DB_USER").expect("missing database username");
    let pass = std::env::var("DB_PASS").expect("missing database password");
    let db = std::env::var("DB_DBNAME").unwrap_or(String::from("postgres"));

    Client::connect(
        &format!("postgresql://{}:{}@{}:{}/{}", user, pass, host, port, db),
        NoTls,
    )
}

pub fn list() -> Result<Vec<Player>, Error> {
    let mut client = get_client()?;

    let players = client
        .query(
            "SELECT id, name, description FROM player.player ORDER BY name",
            &[],
        )?
        .iter()
        .map(|row| Player {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            aliases: vec![],
        })
        .collect();

    Ok(players)
}

pub fn info(name: &str) -> Result<Option<Player>, Error> {
    let mut client = get_client()?;

    let rows = client.query(
        "SELECT id, name, description FROM player.player WHERE name = $1 OR id = (
            SELECT player_id FROM player.player_alias WHERE alias = $1 AND hidden = false
        )",
        &[&name],
    )?;
    if rows.len() == 0 {
        return Ok(None);
    }
    let row = &rows[0];

    let player_id = row.get(0);
    let aliases: Vec<String> = client
        .query(
            "SELECT alias FROM player.player_alias WHERE player_id = $1 AND hidden = false ORDER BY alias",
            &[&player_id],
        )?
        .iter()
        .map(|row| row.get(0))
        .collect();
    let player = Player {
        id: player_id,
        name: row.get(1),
        description: row.get(2),
        aliases,
    };

    Ok(Some(player))
}

pub fn create(name: &str) -> Result<Uuid, Error> {
    let mut client = get_client()?;

    let uuid = client
        .query_one(
            "INSERT INTO player.player(name) VALUES ($1) RETURNING id",
            &[&name],
        )?
        .get(0);

    Ok(uuid)
}

pub fn add_alias(name: &str, aliases: &[&str]) -> Result<u64, Error> {
    let mut client = get_client()?;
    let mut count = 0;

    for alias in aliases {
        count += client.execute(
            "INSERT INTO player.player_alias(player_id, alias) SELECT id, $2 FROM player.player WHERE name = $1",
            &[&name, alias],
        )?;
    }

    Ok(count)
}

pub struct SearchResult {
    pub is_alias: bool,
    pub name: String,
    pub primary_name: String,
}

pub fn search(name: &str) -> Result<Vec<SearchResult>, Error> {
    let mut client = get_client()?;
    let mut results = vec![];

    for row in client.query(
        "
            SELECT false AS is_alias, name, name AS primary_name, levenshtein(name, $1::text) AS diff FROM player.player
            UNION
            SELECT true, a.alias, p.name, levenshtein(a.alias, $1::text) FROM player.player_alias a
            INNER JOIN player.player p ON p.id = a.player_id
            WHERE a.hidden = false
            ORDER BY diff
        ",
        &[&name],
    )? {
        let result = SearchResult {
            is_alias: row.get(0),
            name: row.get(1),
            primary_name: row.get(2),
        };
        results.push(result);
    }

    Ok(results)
}
