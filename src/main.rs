use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "git", about = "the stupid content tracker")]
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

fn main() {
    let args = Cli::from_args();


    println!("Actual functionality coming soon!");
}
