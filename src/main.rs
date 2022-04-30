use anyhow::Result;
use futures::{stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

const CONCURRENCY: usize = 2;

#[tokio::main]
async fn main() -> Result<()> {
    let urls = vec![
        "https://hub.docker.com/v2/repositories/acim/go-reflex/tags/?page=1&page_size=100",
        "https://hub.docker.com/v2/repositories/ectobit/rspamd/tags/?page=1&page_size=100",
    ];

    let client = Client::new();

    let bodies = stream::iter(urls)
        .map(|url| {
            let client = client.clone();
            tokio::spawn(async move {
                let resp = client.get(url).send().await?;
                resp.text().await
            })
        })
        .buffer_unordered(CONCURRENCY);

    bodies
        .for_each(|b| async {
            match b {
                Ok(Ok(s)) => {
                    let resp = serde_json::from_str::<Response<Tag>>(&s);
                    match resp {
                        Ok(r) => println!("{:#?}", r),
                        Err(e) => eprintln!("serde error: {}", e),
                    }
                }
                Ok(Err(e)) => eprintln!("reqwest error: {}", e),
                Err(e) => eprintln!("tokio error: {}", e),
            }
        })
        .await;

    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Response<'a, T> {
    pub count: u64,
    pub next: Option<Cow<'a, str>>,
    pub previous: Option<Cow<'a, str>>,
    pub results: Vec<T>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag<'a> {
    pub creator: u64,
    pub id: u64,
    // pub image_id: ::serde_json::Value,
    // pub images: Vec<Image>,
    pub last_updated: Option<String>,
    pub last_updater: u64,
    pub last_updater_username: &'a str,
    pub name: &'a str,
    pub repository: u64,
    pub full_size: u64,
    pub v2: bool,
    pub tag_status: &'a str,
    pub tag_last_pulled: Option<String>,
    pub tag_last_pushed: Option<String>,
}
