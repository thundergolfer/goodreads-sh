use structopt::StructOpt;
use config;
use oauth1::Token;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::io::stdin;
use url::form_urlencoded;

extern crate dirs;

#[derive(Debug)]
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
    #[structopt(name = "update")]
    /// TODO: Add a help msg here for User
    Update {
        #[structopt(short = "a")]
        all: bool
    },
    #[structopt(name = "auth")]
    Authenticate {

    }
}

#[derive(Serialize, Deserialize)]
struct GoodReadsConfig {
    developer_key: String,
    developer_secret: String,
    access_token: Option<String>,
    access_token_secret: Option<String>,
}

#[derive(Debug)]
struct OAuthAccessToken {
    token: String,
    token_secret: String,
}

fn load_client_config(config_file_path: PathBuf) -> config::Config {
    let mut settings  = config::Config::default();
    settings.merge( config::File::from(config_file_path)).unwrap();
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
        tokens[1] = &tokens[1][1..val_len-1];
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
    let res = client.get(request_token_url)
        .header(
            reqwest::header::AUTHORIZATION,
            oauth1::authorize(
                "GET",
                request_token_url,
                &consumer_token,
                None,
                None,
            )
        )
        .send().unwrap().text().unwrap();
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

        stdin().read_line(&mut answer)
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
        .send().unwrap().text().unwrap();

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

fn client_config_path() -> PathBuf {
    let home_directory: PathBuf = dirs::home_dir()
        .expect("Could not determined home directory.");
    let mut config_file_path : PathBuf = PathBuf::new();
    config_file_path.push(home_directory);
    config_file_path.push(".goodreads.toml");
    config_file_path
}

fn run_command(args: &Cli, app_config: &GoodReadsConfig) {
    match *args {
        Cli::Update { .. } => {
            let url = "https://www.goodreads.com/shelf/add_to_shelf.xml";
            let app_params = vec![
                (String::from("name"), String::from("to-read")),
                (String::from("book_id"), String::from("631932"))
            ];
            let mut app_params_map: HashMap<&str, Cow<str>> = HashMap::new();
            for key_val in app_params.iter() {
                app_params_map.insert(key_val.0.as_str(), Cow::from(key_val.1.clone()));
            }
            println!("{:?}", app_params_map);
            let oauth_header_str = oauth1::authorize(
                "POST",
                url,
                &Token::new(
                    app_config.developer_key.clone(),
                    app_config.developer_secret.clone()
                ),
                Some(&Token::new(
                    app_config.access_token.as_ref().expect("Access token should never be None here").clone(),
                    app_config.access_token_secret.as_ref().expect("Access token should never be None here").clone()
                )),
                Some(app_params_map),
            );
            let mut form_params = oauth_header_string_to_form_data(
                &oauth_header_str
            );
            form_params.extend(app_params);
            println!("{:?}", form_params);

            let client = reqwest::Client::new();
            let res = client.post(url)
                .form(&form_params)
                .send().unwrap();

            println!("{:?}", res);
        },
        Cli::Book { } => println!("'book' not yet implemented"),
        Cli::Author { } => println!("'author' not yet implemented."),
        Cli::Authenticate { } => println!("Already authenticated.")
    }
}

fn main() {
    let args = Cli::from_args();

    let cfg: config::Config = load_client_config(client_config_path());

    let dev_key : String = cfg.get_str("developer_key").unwrap();
    let dev_secret: String = cfg.get_str("developer_secret").unwrap();
    let access_token_res: Result<String, _> = cfg.get_str("access_token");
    let access_token_secret_res: Result<String, _> = cfg.get_str("access_token_secret");

    // TODO(Jonathon): Check for both access_token and access_token_secret
    match access_token_res {
        Ok(access_token) => {
            let app_config = GoodReadsConfig {
                developer_secret: dev_secret,
                developer_key: dev_key,
                access_token_secret: Some(access_token_secret_res.unwrap()),
                access_token: Some(access_token),
            };
            run_command(&args, &app_config);
        }
        Err(_err) => {
            match args {
                Cli::Authenticate { } => {
                    let oauth_access_token = get_oauth_token(dev_key, dev_secret);
                    add_access_token_to_config(client_config_path(), &oauth_access_token)
                }
                _ => println!("OAuth not set up. Please run: goodreads-sh auth")
            }
        }
    }
}
