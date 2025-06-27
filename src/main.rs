pub mod command;
pub mod database;
pub mod model;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    match command::do_command(&args[1..]).await {
        Ok(s) => print!("{}", s),
        Err(e) => panic!("{}", e),
    };
}
