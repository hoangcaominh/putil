use std::collections::VecDeque;

use crate::database::Database;

pub enum Command {
    NoOp,
    Help,
    List,
    Info,
    Create,
    AddAlias,
    Search,
}

impl Command {
    pub fn parse(s: &str) -> Command {
        match s {
            "h" | "help" => Command::Help,
            "l" | "ls" | "list" => Command::List,
            "i" | "info" => Command::Info,
            "c" | "create" => Command::Create,
            "a" | "addalias" => Command::AddAlias,
            "s" | "search" => Command::Search,
            _ => Command::NoOp,
        }
    }

    pub fn print_help(command: Command) -> String {
        match command {
            Command::List => {
                print_msg_queue(vec!["Usage: l|ls|list".into(), "List all players".into()])
            }
            Command::Info => print_msg_queue(vec![
                "Usage: i|info <name|alias>".into(),
                "Show information about a player".into(),
            ]),
            Command::Create => print_msg_queue(vec![
                "Usage: c|create <name>".into(),
                "Create a new player entry".into(),
            ]),
            Command::AddAlias => print_msg_queue(vec![
                "Usage: a|addalias <name> <alias1> [<alias2>..]".into(),
                "Add one or more aliases of a player".into(),
            ]),
            Command::Search => print_msg_queue(vec![
                "Usage: s|search <name|alias>".into(),
                "Search players".into(),
            ]),
            Command::Help | Command::NoOp => print_msg_queue(vec![
                "Usage: [l|i|c|a|s] [<arguments>]".into(),
                "Issue h or help followed by a command to show more details".into(),
            ]),
        }
    }
}

fn print_msg_queue(queue: Vec<String>) -> String {
    queue.join("\n")
}

pub async fn do_command(args: &[String]) -> Result<String, String> {
    let db = match Database::new().await {
        Ok(val) => val,
        Err(e) => {
            return Err(format!("Failed to connect to database: {}", e));
        },
    };
    let mut args = VecDeque::from_iter(args);

    let command = match args.pop_front() {
        Some(val) => val,
        None => return Ok(Command::print_help(Command::NoOp)),
    };

    let mut msg_queue = vec![];
    match Command::parse(command) {
        Command::List => match db.list().await {
            Ok(players) => {
                for player in players {
                    msg_queue.push(format!("{}", &player.name));
                }
            }
            Err(e) => return Err(format!("Failed to list players: {}", e)),
        },
        Command::Info => {
            let name = match args.pop_front() {
                Some(val) => val,
                None => {
                    return Err(print_msg_queue(vec![
                        "Invalid arguments".into(),
                        Command::print_help(Command::Info),
                    ]));
                }
            };
            match db.info(name).await {
                Ok(res) => match res {
                    Some(player) => {
                        msg_queue.push("Player info:".into());
                        msg_queue.push(format!("Name: {}", player.name));
                        msg_queue.push(format!("Description: {}", player.description));
                        if player.aliases.len() > 0 {
                            msg_queue.push(format!("Aliases: {}", player.aliases.join(", ")));
                        }
                    }
                    None => msg_queue.push(format!("No player called {} was found\n", name)),
                },
                Err(e) => return Err(format!("Failed to search for player {}: {}", name, e)),
            }
        }
        Command::Help => match args.pop_front() {
            Some(subcommand) => return Ok(Command::print_help(Command::parse(subcommand))),
            None => return Ok(Command::print_help(Command::NoOp)),
        },
        Command::Create => {
            let name = match args.pop_front() {
                Some(val) => val,
                None => {
                    return Err(print_msg_queue(vec![
                        "Invalid arguments".into(),
                        Command::print_help(Command::Create),
                    ]));
                }
            };
            match db.create(name).await {
                Ok(id) => msg_queue.push(format!("Created player {} with id {}", name, id)),
                Err(e) => return Err(format!("Failed to create player {}: {}", name, e)),
            }
        }
        Command::AddAlias => {
            let name = match args.pop_front() {
                Some(val) => val,
                None => {
                    return Err(print_msg_queue(vec![
                        "Invalid arguments".into(),
                        Command::print_help(Command::AddAlias),
                    ]));
                }
            };
            let aliases: Vec<&str> = args.iter().map(|alias| alias.trim()).collect();
            if aliases.is_empty() {
                return Err(print_msg_queue(vec![
                    format!("Missing aliases for {}", name),
                    Command::print_help(Command::AddAlias),
                ]));
            }
            match db.add_alias(name, &aliases).await {
                Ok(count) => {
                    if count > 0 {
                        msg_queue.push(format!("Added {} aliased for player {}", count, name));
                    } else {
                        msg_queue.push(format!("No player called {} was found", name));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Failed to add aliases for player {}: {}",
                        name, e
                    ));
                }
            }
        }
        Command::Search => {
            let name = match args.pop_front() {
                Some(val) => val,
                None => {
                    return Err(print_msg_queue(vec![
                        "Invalid arguments".into(),
                        Command::print_help(Command::Search),
                    ]));
                }
            };
            match db.search(name).await {
                Ok(results) => {
                    println!("Found {} results:", results.len());
                    for result in results {
                        if result.is_alias {
                            msg_queue.push(format!(
                                "{} - Alias of {}",
                                result.name, result.primary_name
                            ));
                        } else {
                            msg_queue.push(format!("{}", result.name));
                        }
                    }
                }
                Err(e) => return Err(format!("Failed to search for player {}: {}", name, e)),
            }
        }
        Command::NoOp => {
            return Err(print_msg_queue(vec![
                "Invalid command".into(),
                Command::print_help(Command::NoOp),
            ]));
        }
    }

    Ok(print_msg_queue(msg_queue))
}
