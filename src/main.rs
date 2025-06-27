use std::process;

pub mod command;
pub mod database;
pub mod model;

fn main() {
    dotenv::dotenv().ok();
    if let Err(e) = database::get_client() {
        eprintln!("{}", e);
        process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    match command::do_command(&args[1..]) {
        Ok(s) => print!("{}", s),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
