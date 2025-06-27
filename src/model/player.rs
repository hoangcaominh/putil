use uuid::Uuid;

#[derive(Debug)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
}
