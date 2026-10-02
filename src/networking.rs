use aidoku::{
    alloc::{format, string::String},
    imports::net::Request,
    Result,
};
use crate::auth;

const BASE_URL: &str = "https://comics-tracker.net";

pub fn create_get_request(endpoint: &str) -> Result<Request> {
    let url = if endpoint.starts_with("http") {
        String::from(endpoint)
    } else {
        format!("{}{}", BASE_URL, endpoint)
    };

    let mut req = Request::get(url)?;

    if let Some(token) = auth::get_firebase_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }

    Ok(req)
}
