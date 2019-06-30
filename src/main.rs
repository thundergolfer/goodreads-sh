use structopt::StructOpt;
use config;
use oauth1::Token;
use reqwest::Client;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::stdin;
use url::form_urlencoded;

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

#[derive(Debug)]
struct OAuthAccessToken {
    token: String,
    token_secret: String,
}

fn load_client_config() -> config::Config {
    let home_directory: PathBuf = dirs::home_dir()
        .expect("Could not determined home directory.");
    let mut config_file_path = PathBuf::new();
    config_file_path.push(home_directory);
    config_file_path.push(".goodreads");

    let mut settings  = config::Config::default();
    settings.merge( config::File::from(config_file_path)).unwrap();
    settings
}

/// Goodreads.com weirdly wants the OAuth content in a query string, so
/// convert convert the header into a valid query string
fn oauth_header_string_into_query_string(oauth_header: &str) -> String {
    let oauth_header = &oauth_header[6..]; // remove the "OAuth " prefix

    let mut tokens: Vec<&str>;
    let mut cleaned_pairs: Vec<String> = Vec::new();

    for key_val_pair in oauth_header.split(", ") {
        tokens = key_val_pair.split("=").collect();
        // Remove quotes from value
        let val_len = tokens[1].len();
        tokens[1] = &tokens[1][1..val_len-1];
        // Rejoin and append
        let cleaned_pair = tokens.join("=").to_string();
        cleaned_pairs.push(cleaned_pair);
    }

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

fn main() {
    let args = Cli::from_args();
    let cfg: config::Config = load_client_config();

    let dev_key : String = cfg.get_str("developer_key").unwrap();
    let dev_secret: String = cfg.get_str("developer_secret").unwrap();
    let oauth_access_token = get_oauth_token(dev_key, dev_secret);

    println!("{:?}", oauth_access_token);
}
