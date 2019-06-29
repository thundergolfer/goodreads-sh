use structopt::StructOpt;
use config;
use std::collections::HashMap;
use std::path::PathBuf;

extern crate dirs;

#[derive(StructOpt)]
#[structopt(name = "goodreads-sh", about = "CLI interface to Goodreads.com")]
enum Cli {
    #[structopt(name = "book")]
    /// TODO: Add a help msg here for Book
    Book {
    },
    #[structopt(name = "author")]
    /// TODO: Add a help msg here for Author
    Author {
    },
    #[structopt(name = "user")]
    /// TODO: Add a help msg here for User
    User {
        #[structopt(short = "a")]
        all: bool
    }
}

fn load_client_config() -> config::Config {
    let home_directory: PathBuf = dirs::home_dir()
        .expect("Could not determined home directory.");
    let mut config_file_path = PathBuf::new();
    config_file_path.push(home_directory);
    config_file_path.push(".goodreads");

    let mut settings  = config::Config::default();
    settings.merge( config::File::from(config_file_path)).unwrap();
}

fn main() {
    let args = Cli::from_args();
    let config = load_client_config();


    println!("{:?}", settings.try_into::<HashMap<String, String>>().unwrap());


    println!("Actual functionality coming soon!");
}
