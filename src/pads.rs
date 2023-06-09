use anyhow::Result;
use chrono::Utc;
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::Deserialize;
use url::Url;

use crate::Link;

#[derive(Debug, Clone)]
pub struct Pad {
    pub link: Link,

    pub title: String,

    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,

    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct Info {
    #[serde(rename = "title")]
    pub title: String,

    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "viewcount")]
    pub view_count: u64,

    #[serde(rename = "createtime")]
    pub create_time: chrono::DateTime<Utc>,

    #[serde(rename = "updatetime")]
    pub update_time: chrono::DateTime<Utc>,
}

impl Pad {
    pub async fn fetch(link: &Link) -> Result<Pad> {
        let info: Info = reqwest::get(format!("https://{}/{}/info", link.server, link.name)).await?
            .json().await?;

        let content = reqwest::get(format!("https://{}/{}/download", link.server, link.name)).await?
            .text().await?;

        return Ok(Pad {
            link: link.to_owned(),
            title: info.title,
            create_time: info.create_time,
            update_time: info.update_time,
            content,
        });
    }

    pub fn crawl(&self) -> Result<Vec<String>> {
        let parser = Parser::new_ext(&self.content, Options::all())
            .into_offset_iter();

        let mut links = Vec::new();
        for (event, _) in parser {
            if let Event::Start(Tag::Link(_link_type, link_dest, _link_title)) = event {
                let mut link_dest = Url::options()
                    .base_url(Some(&self.link.as_url()))
                    .parse(&link_dest)?;
                link_dest.set_fragment(None);

                links.push(link_dest.to_string());
            }
        }

        return Ok(links);
    }
}