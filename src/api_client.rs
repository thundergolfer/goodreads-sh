use serde::{Deserialize, Serialize};

use super::models;
use reqwest::{Client, Response, Method};
use reqwest::header::HeaderValue;
use oauth_client;
use std::collections::HashMap;
use std::hash::Hash;
use std::collections::hash_map::RandomState;
use std::borrow::Cow;

mod goodreads_api_endpoints {
    pub const USER_ID: &'static str = "https://www.goodreads.com/api/auth_user";
    pub const LIST_SHELF: &'static str = "https://www.goodreads.com/review/list?v=2";
    pub const ADD_TO_SHELF: &'static str = "https://www.goodreads.com/shelf/add_to_shelf.xml";
    pub const UPDATE_STATUS: &'static str = "https://www.goodreads.com/user_status.xml";
}

#[derive(Serialize, Deserialize)]
pub struct GoodreadsApiClientAuth {
    developer_key: String,
    developer_secret: String,
    oauth_access_token: String,
    oauth_access_token_secret: String,
}


pub struct GoodreadsApiClient {
    pub auth: GoodreadsApiClientAuth,
    pub user_id: u32,
    pub client: Client,
}

impl GoodreadsApiClient {
    pub fn user_id(&self) -> usize {
        // TODO(Jonathon): Implement
        10000
    }

    pub fn update_status(
        &self,
        book: Option<&models::Book>,
        page: Option<u32>,
        percent: Option<Percentage>,
        body: Option<String>,
    ) -> Result<(), String> {
        if body.is_some() {
            return Err("Sending status updates with a 'body' is not yet supported.".to_owned());
        } else if book.is_none() {
            return Err("Sending status updates without a book is not yet supported.".to_owned());
        } else if page.is_some() && percent.is_some() {
            return Err("Cannot specify both 'page' and 'percent' progress indicators. Choose 1.".to_owned());
        }

        let mut req_params = HashMap::new();
        if page.is_some() {
            let page = page.unwrap().to_string();
            let _ = req_params.insert(Cow::from("user_status[page]"), Cow::from(page));
        } else if percent.is_some() {
            let percent = Percentage::unwrap(percent.unwrap()).to_string();
            let _ = req_params.insert(Cow::from("user_status[percent]"), Cow::from(percent));
        }
        let book_id = book.unwrap().id.to_string();
        let _ = req_params.insert(Cow::from("user_status[book]"), Cow::from(book_id));

        let resp = make_oauthd_request(
            self,
            reqwest::Method::POST,
            goodreads_api_endpoints::UPDATE_STATUS,
            req_params,
        );

        match resp {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("Request failed: {}", err))
        }
    }

    pub fn new(
        user_id: u32,
        developer_key: &str,
        developer_secret: &str,
        oauth_access_token: &str,
        oauth_access_token_secret: &str,
    ) -> GoodreadsApiClient {
        let auth = GoodreadsApiClientAuth {
            developer_key: developer_key.to_string(),
            developer_secret: developer_secret.to_string(),
            oauth_access_token: oauth_access_token.to_string(),
            oauth_access_token_secret: oauth_access_token_secret.to_string(),
        };

        GoodreadsApiClient {
            auth,
            user_id,
            client: Client::new(),
        }
    }
}

fn make_oauthd_request(
    gr_client: &GoodreadsApiClient,
    method: reqwest::Method,
    url: &str,
    req_params: oauth_client::ParamList,
) -> Result<reqwest::Response, reqwest::Error> {
    let consumer = oauth_client::Token::new(
        gr_client.auth.developer_key.clone(),
        gr_client.auth.developer_secret.clone(),
    );
    let access = oauth_client::Token::new(
        gr_client.auth.oauth_access_token.clone(),
            gr_client.auth.oauth_access_token_secret.clone(),
    );
    let (header, body) = oauth_client::authorization_header(
        method.as_str(),
        url,
        &consumer,
        Some(&access),
        Some(&req_params),
    );

    let req = gr_client.client
        .get(url)
        .header(reqwest::header::AUTHORIZATION, header)
        .header(
            reqwest::header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
        )
        .body(body);
    req.send()
}

#[derive(Clone,Copy,Debug)]
pub struct Percentage(u8);

impl Percentage {
    pub fn new(x: u8) -> Option<Percentage> {
        if x <= 100 {
            Some(Percentage(x))
        } else {
            None
        }
    }

    pub fn unwrap(p: Percentage) -> u8 {
        p.0
    }
}