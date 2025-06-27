use tokio_postgres::{Client, Error, NoTls};
use uuid::Uuid;

use crate::model::Player;

pub struct SearchResult {
    pub is_alias: bool,
    pub name: String,
    pub primary_name: String,
}

pub struct Database {
    client: Client,
}

impl Database {
    pub async fn new() -> Result<Database, Error> {
        let host = std::env::var("DB_HOST").expect("missing database host");
        let port = std::env::var("DB_PORT").unwrap_or(String::from("5432"));
        let user = std::env::var("DB_USER").expect("missing database username");
        let pass = std::env::var("DB_PASS").expect("missing database password");
        let db = std::env::var("DB_DBNAME").unwrap_or(String::from("postgres"));

        let (client, connection) = tokio_postgres::connect(
            &format!("postgresql://{}:{}@{}:{}/{}", user, pass, host, port, db),
            NoTls,
        )
        .await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(Database { client })
    }
}

impl Database {
    pub async fn list(&self) -> Result<Vec<Player>, Error> {
        let players = self.client
            .query(
                "SELECT id, name, description FROM player.player ORDER BY name",
                &[],
            ).await?
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

    pub async fn info(&self, name: &str) -> Result<Option<Player>, Error> {
        let rows = self.client.query(
            "SELECT id, name, description FROM player.player WHERE name = $1 OR id = (
                SELECT player_id FROM player.player_alias WHERE alias = $1 AND hidden = false
            )",
            &[&name],
        ).await?;
        if rows.len() == 0 {
            return Ok(None);
        }
        let row = &rows[0];

        let player_id = row.get(0);
        let aliases: Vec<String> = self.client
            .query(
                "SELECT alias FROM player.player_alias WHERE player_id = $1 AND hidden = false ORDER BY alias",
                &[&player_id],
            ).await?
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

    pub async fn create(&self, name: &str) -> Result<Uuid, Error> {
        let uuid = self.client
            .query_one(
                "INSERT INTO player.player(name) VALUES ($1) RETURNING id",
                &[&name],
            ).await?
            .get(0);

        Ok(uuid)
    }

    pub async fn add_alias(&self, name: &str, aliases: &[&str]) -> Result<u64, Error> {
        let mut count = 0;

        for alias in aliases {
            count += self.client.execute(
                "INSERT INTO player.player_alias(player_id, alias) SELECT id, $2 FROM player.player WHERE name = $1",
                &[&name, alias],
            ).await?;
        }

        Ok(count)
    }

    pub async fn search(&self, name: &str) -> Result<Vec<SearchResult>, Error> {
        let mut results = vec![];

        for row in self.client.query(
            "
                SELECT false AS is_alias, name, name AS primary_name, levenshtein(name, $1::text) AS diff FROM player.player
                UNION
                SELECT true, a.alias, p.name, levenshtein(a.alias, $1::text) FROM player.player_alias a
                INNER JOIN player.player p ON p.id = a.player_id
                WHERE a.hidden = false
                ORDER BY diff
            ",
            &[&name],
        ).await? {
            let result = SearchResult {
                is_alias: row.get(0),
                name: row.get(1),
                primary_name: row.get(2),
            };
            results.push(result);
        }

        Ok(results)
    }
}
