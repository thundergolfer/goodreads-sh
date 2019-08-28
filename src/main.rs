use config;
use oauth1::Token;
use oauth_client;
use reqwest::header::HeaderValue;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::io::stdin;
use std::path::PathBuf;
use structopt::StructOpt;
use url::form_urlencoded;
use std::process::exit;

extern crate dirs;

mod api_client;
pub mod models;
mod ux;

#[derive(Debug, StructOpt)]
#[structopt(name = "goodreads-sh", about = "CLI interface to Goodreads.com")]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
enum Cli {
    #[structopt(name = "add-to-shelf")]
    /// Add a book to an existing shelf [In Progress]
    AddToShelf {},
    #[structopt(name = "me")]
    /// Show your User ID
    Me {},
    #[structopt(name = "new")]
    /// Tell Goodreads you've started a new book
    New {
        #[structopt(short = "t", long = "title")]
        title: Option<String>,
    },
    #[structopt(name = "update")]
    /// Update progress on a book you're currently reading
    Update {
        #[structopt(short = "t", long = "title")]
        title: Option<String>,
    },
    #[structopt(name = "finished")]
    /// Tell Goodreads you've finished a book you're currently reading
    Finished {},
    /// Setup OAuth for the CLI (1 time only)
    #[structopt(name = "auth")]
    Authenticate {},
}

#[derive(Serialize, Deserialize)]
struct GoodReadsConfig {
    developer_key: String,
    developer_secret: String,
    access_token: Option<String>,
    access_token_secret: Option<String>,
    user_id: Option<u32>,
}

#[derive(Debug)]
struct OAuthAccessToken {
    token: String,
    token_secret: String,
}

fn load_client_config(config_file_path: PathBuf) -> config::Config {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::from(config_file_path))
        .unwrap();
    settings
}

/// Goodreads.com wants OAuth content in form data for POSTs (Is this typical?)
fn oauth_header_string_to_form_data(oauth_header: &str) -> Vec<(String, String)> {
    let oauth_header = &oauth_header[6..]; // remove the "OAuth " prefix
    let mut tokens: Vec<&str>;
    let mut form_data: Vec<(String, String)> = Vec::new();

    for key_val_pair in oauth_header.split(", ") {
        tokens = key_val_pair.split("=").collect();
        // Remove quotes from value
        let val_len = tokens[1].len();
        tokens[1] = &tokens[1][1..val_len - 1];
        form_data.push((tokens[0].to_owned(), tokens[1].to_owned()));
    }

    form_data
}

/// Goodreads.com weirdly wants the OAuth content in a query string, so
/// convert convert the header into a valid query string
fn oauth_header_string_into_query_string(oauth_header: &str) -> String {
    let cleaned_pairs: Vec<String> = oauth_header_string_to_form_data(oauth_header)
        .into_iter()
        .map(|pair| format!("{}={}", pair.0, pair.1))
        .collect();

    cleaned_pairs.join("&")
}

fn get_oauth_token(client_id: String, client_secret: String) -> OAuthAccessToken {
    let auth_url = "https://www.goodreads.com/oauth/authorize";
    let request_token_url = "https://www.goodreads.com/oauth/request_token";
    let token_url = "https://www.goodreads.com/oauth/access_token";

    let mut extra_params: HashMap<&str, Cow<str>> = HashMap::new();
    let consumer_token = Token::new(client_id, client_secret);
    extra_params.insert("oauth_callback", Cow::from("oob"));

    let client = Client::new();
    let res = client
        .get(request_token_url)
        .header(
            reqwest::header::AUTHORIZATION,
            oauth1::authorize("GET", request_token_url, &consumer_token, None, None),
        )
        .send()
        .unwrap()
        .text()
        .unwrap();
    let params: HashMap<String, String> = form_urlencoded::parse(&res.as_bytes())
        .into_owned()
        .collect();
    let request_token = params.get("oauth_token").unwrap();
    let request_token_secret = params.get("oauth_token_secret").unwrap();

    let constructed_auth_url = format!("{}?oauth_token={}", auth_url, request_token);

    println!("Visit this URL in the browser: {}", constructed_auth_url);

    println!("Have you authorised in the browser?: y/n");

    loop {
        let mut answer = String::new();

        stdin()
            .read_line(&mut answer)
            .expect("Failed to read the line");

        if answer.trim() == "y" {
            println!("Thankyou. Finishing authentication process...");
            break;
        }
    }

    let oauth_headers = oauth1::authorize(
        "GET",
        token_url,
        &consumer_token,
        Some(&Token::new(request_token, request_token_secret)),
        None,
    );

    let oauth_query_string = oauth_header_string_into_query_string(&oauth_headers);

    let token_url_with_oauth = format!("{}?{}", token_url, oauth_query_string);

    let res = client
        .get(&token_url_with_oauth)
        .send()
        .unwrap()
        .text()
        .unwrap();

    let access_token_params: HashMap<String, String> = form_urlencoded::parse(&res.as_bytes())
        .into_owned()
        .collect();
    let access_token = access_token_params.get("oauth_token").unwrap();
    let access_token_secret = access_token_params.get("oauth_token_secret").unwrap();

    OAuthAccessToken {
        token: access_token.to_owned(),
        token_secret: access_token_secret.to_owned(),
    }
}

fn add_access_token_to_config(client_config_path: PathBuf, oauth_access_token: &OAuthAccessToken) {
    let value = fs::read_to_string(client_config_path.clone()).unwrap();

    let mut config: GoodReadsConfig = toml::from_str(&value).unwrap();
    config.access_token = Some(oauth_access_token.token.clone());
    config.access_token_secret = Some(oauth_access_token.token_secret.clone());

    let toml = toml::to_string(&config).unwrap();
    fs::write(client_config_path, toml).unwrap();
}

fn add_user_id_to_config(
    client_config_path: PathBuf,
    gr_client: &api_client::GoodreadsApiClient,
) -> Result<(), String> {
    let value = fs::read_to_string(client_config_path.clone()).unwrap();
    let mut config: GoodReadsConfig = toml::from_str(&value).unwrap();
    let user_id = gr_client.user_id();
    match user_id {
        Ok(id) => {
            config.user_id = Some(id);
            let toml = toml::to_string(&config).unwrap();
            fs::write(client_config_path, toml).unwrap();
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn client_config_path() -> PathBuf {
    let home_directory: PathBuf = dirs::home_dir().expect("Could not determined home directory.");
    let mut config_file_path: PathBuf = PathBuf::new();
    config_file_path.push(home_directory);
    config_file_path.push(".goodreads.toml");
    config_file_path
}

fn run_command(
    args: &Cli,
    app_config: &GoodReadsConfig,
    gr_client: &api_client::GoodreadsApiClient,
) {
    match args {
        Cli::AddToShelf {} => {
            // TODO(Jonathon): Stop hardcoding these
            let res = gr_client.add_to_shelf(9282, "to-read");
            match res {
                Ok(_) => println!("Added ✅"),
                Err(err) => println!("fuck: {}", err),
            }
        }
        Cli::New { title } => {
            let mut answer = String::new();
            let titleQuery = title.as_ref().unwrap_or_else(|| {
                println!("Enter a title to search:");
                stdin()
                    .read_line(&mut answer)
                    .expect("Failed to read your input");
                &answer
            });
            let search_results  = gr_client.search_books(&titleQuery, "title");
            println!("Not yet implemented")
        }
        Cli::Finished {} => {
            let res = gr_client.list_shelf("currently-reading")
                .and_then(|shelf_xml| models::parse_shelf(&shelf_xml).map_err(|err| err.to_string()))
                .and_then(|shelf| {
                    for (i, book) in shelf.books.iter().enumerate() {
                        println!("{}. {}", i + 1, book);
                    }
                    println!("\nWhich one have you finished?");
                    let choice = ux::get_choice(1, shelf.books.len() as u32);
                    let book_to_update = shelf
                        .books
                        .get((choice as usize) - 1)
                        .expect("Should never here access an invalid index");
                    Ok(book_to_update.id)
                })
                .and_then(|id| {
                    gr_client.remove_from_shelf(id, "currently-reading")?;
                    Ok(id)
                })
                .and_then(|id| gr_client.add_to_shelf(id, "read"));
            match res {
                Ok(_) => println!("✅ Nice work!"),
                Err(err) => print!("Error: {}", err)
            }
        }
        Cli::Update { title } => {
            let res = gr_client.list_shelf("currently-reading");
            match res {
                Ok(shelf_xml) => {
                    let shelf: models::Shelf = models::parse_shelf(&shelf_xml).unwrap();

                    let book_to_update = match title {
                        Some(t) => {
                            // TODO(Jonathon): This doesn't handle 0-len shelves
                            let closest = ux::select_by_edit_distance(&shelf, &t);
                            let b = closest.0.unwrap();
                            let dist_from_target = closest.1.unwrap();
                            // If what user typed and the closest book title are exactly alike (0) or only 1 char difference (1),
                            // then just assume a match. Otherwise, check that we are updating the right book.
                            if dist_from_target < 2 {
                                b
                            } else {
                                println!(
                                    "❓: You typed '{}'. The closest book found in your 'currently-reading' shelf is '{}'. Is that a match?: y/n",
                                    t,
                                    b.title.clone(),
                                );
                                let confirmed = ux::get_confirm();
                                if confirmed {
                                    b
                                } else {
                                    println!("Try retyping the book title.");
                                    std::process::exit(0);
                                }
                            }
                        },
                        None => {
                            for (i, book) in shelf.books.iter().enumerate() {
                                println!("{}. {}", i + 1, book);
                            }
                            println!("Choose a book to update progress on:");
                            // TODO(Jonathon): Handle case where shelf has no books
                            let choice = ux::get_choice(1, shelf.books.len() as u32);
                            shelf.books
                                .get((choice as usize) - 1)
                                .expect("Should never here access an invalid index")
                                .clone()
                        }
                    };

                    match book_to_update.num_pages {
                        Some(val) => {
                            println!("What page are you on now? (Max page is {}):", val);
                            let current_page = ux::get_choice(1, val);
                            println!("You're on {}!", current_page);
                            gr_client
                                .update_status(Some(&book_to_update), Some(current_page), None, None)
                                .unwrap();
                        }
                        None => {
                            println!("What page are you on now?:");
                            let current_page = ux::get_choice(1, 10_000);
                            gr_client
                                .update_status(Some(&book_to_update), Some(current_page), None, None)
                                .unwrap();
                        }
                    }
                }
                Err(err) => println!("fuck: {}", err),
            }
        }
        Cli::Me {} => {
            let user_id = gr_client.user_id();
            match user_id {
                Ok(id) => println!("Your user id is: {}", id),
                Err(err) => println!("Error: {}", err),
            }
        }
        Cli::Authenticate {} => println!("Already authenticated."),
    }
}

fn main() {
    let args = Cli::from_args();

    let cfg: config::Config = load_client_config(client_config_path());

    let dev_key: String = cfg.get_str("developer_key").unwrap();
    let dev_secret: String = cfg.get_str("developer_secret").unwrap();
    let access_token_res: Result<String, _> = cfg.get_str("access_token");
    let access_token_secret_res: Result<String, _> = cfg.get_str("access_token_secret");
    let user_id_res: Result<i64, _> = cfg.get_int("user_id");

    // TODO(Jonathon): Check for both access_token and access_token_secret
    match access_token_res {
        Ok(access_token) => {
            let user_id = user_id_res.unwrap() as u32;
            let access_token_secret = access_token_secret_res.unwrap();
            let app_config = GoodReadsConfig {
                developer_secret: dev_secret.clone(),
                developer_key: dev_key.clone(),
                access_token_secret: Some(access_token_secret.clone()),
                access_token: Some(access_token.clone()),
                user_id: Some(user_id),
            };
            let gr_client = api_client::GoodreadsApiClient::new(
                user_id,
                &dev_key,
                &dev_secret,
                &access_token,
                &access_token_secret,
            );

            run_command(&args, &app_config, &gr_client);
        }
        Err(_err) => {
            match args {
                Cli::Authenticate {} => {
                    let oauth_access_token = get_oauth_token(dev_key.clone(), dev_secret.clone());
                    println!("Adding oauth access token to config...");
                    add_access_token_to_config(client_config_path(), &oauth_access_token);
                    let gr_client = api_client::GoodreadsApiClient::new(
                        0, // TODO(Jonathon): This is an invalid value, though it will never be used. Still should clean this up.
                        &dev_key,
                        &dev_secret,
                        &oauth_access_token.token,
                        &oauth_access_token.token_secret,
                    );
                    println!("Adding your user id to config...");
                    match add_user_id_to_config(client_config_path(), &gr_client) {
                        Ok(_) => println!("✅"),
                        Err(err) => println!("Error: {}", err),
                    }
                }
                _ => println!("OAuth not set up. Please run: goodreads-sh auth"),
            }
        }
    }
}
