use once_cell::sync::Lazy;
use std::env;

pub static USERNAME: Lazy<String> =
    Lazy::new(|| env::var("USERNAME").expect("USERNAME environment variable not set"));

pub static PASSWORD: Lazy<String> =
    Lazy::new(|| env::var("PASSWORD").expect("PASSWORD environment variable not set"));

pub static SERVER: Lazy<String> =
    Lazy::new(|| env::var("SERVER").expect("SERVER environment variable not set"));

pub static DB_URL: Lazy<String> =
    Lazy::new(|| env::var("SURREALDB_URL").expect("SURREALDB_URL environment variable not set"));

pub static DB_USERNAME: Lazy<String> =
    Lazy::new(|| env::var("SURREALDB_USER").expect("SURREALDB_USER environment variable not set"));

pub static DB_PASSWORD: Lazy<String> =
    Lazy::new(|| env::var("SURREALDB_PASS").expect("SURREALDB_PASS environment variable not set"));
