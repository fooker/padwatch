#![feature(try_blocks)]
#![feature(hash_drain_filter)]

use std::collections::{HashMap, HashSet, LinkedList};
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use sha2::{Digest, Sha256};
use sha2::digest::Output;
use structopt::StructOpt;
use url::Url;

use crate::config::Config;
use crate::notify::Notifier;
use crate::pads::Pad;
use crate::repo::Repo;

mod config;
mod pads;
mod repo;
mod notify;


#[derive(Debug, StructOpt)]
#[structopt(name = "padwatch", about = "Monitor a cloud of markdown pads for changes.")]
struct Opt {
    #[structopt(short, long, default_value = "config.toml")]
    config: PathBuf,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Link {
    pub server: String,
    pub name: String,
}

impl Link {
    pub fn new(server: String, name: String) -> Self {
        return Self { server, name };
    }

    pub fn from_url(servers: &HashSet<String>, url: &str) -> Option<Self> {
        for server in servers {
            if let Some(name) = url.strip_prefix(&format!("https://{}/", server)) {
                return Some(Self {
                    server: server.to_string(),
                    name: name.to_string(),
                });
            }
        }

        return None;
    }

    fn as_url(&self) -> Url {
        return Url::parse(&self.to_string())
            .expect("Link is not a valid URL");
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "https://{}/{}", self.server, self.name);
    }
}

#[derive(Debug)]
enum Tracker {
    Quiescent {
        hash: Output<Sha256>,
    },

    Vivacious {
        last_updated: Instant,
        last_hash: Output<Sha256>,
    },
}

impl Tracker {
    pub fn from_new(last: &str) -> Self {
        let last_hash = Self::hash(&last);

        return Self::Vivacious {
            last_updated: Instant::now(),
            last_hash,
        };
    }

    pub fn from_existing(orig: &str, last: &str) -> Self {
        let orig_hash = Self::hash(&orig);
        let last_hash = Self::hash(&last);

        return if orig_hash == last_hash {
            Self::Quiescent { hash: orig_hash }
        } else {
            Self::Vivacious {
                last_updated: Instant::now(),
                last_hash,
            }
        };
    }

    pub fn update(&mut self, content: &str) -> &mut Self {
        let hash = Self::hash(content);

        match self {
            Self::Quiescent { hash: orig_hash } => {
                if hash != *orig_hash {
                    *self = Self::Vivacious {
                        last_updated: Instant::now(),
                        last_hash: hash,
                    }
                }
            }

            Self::Vivacious { last_hash, last_updated, .. } => {
                if hash != *last_hash {
                    *last_updated = Instant::now();
                    *last_hash = hash;
                }
            }
        }

        return self;
    }

    pub fn quiesce(&mut self, cool_down: Duration) -> bool {
        if let Self::Vivacious { last_updated, last_hash, .. } = self {
            if Instant::now().duration_since(*last_updated) > cool_down {
                *self = Self::Quiescent {
                    hash: last_hash.clone(),
                };
                return true;
            }
        }

        return false;
    }

    fn hash(content: &str) -> Output<Sha256> {
        return sha2::Sha256::new()
            .chain_update(content)
            .finalize();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    let config = Config::load(opt.config).await?;

    let mut notifier = Notifier::connect(
        &config.notify.username,
        &config.notify.password,
        &config.notify.room,
    ).await?;

    let mut known = config.crawl.seeds.iter()
        .map(|seed| Link::from_url(&config.crawl.servers, seed)
            .ok_or_else(|| anyhow::anyhow!("Seed link not valid: {}", seed)))
        .collect::<Result<HashSet<_>>>()?;

    let mut repo = Repo::open(config.repo.path).await?;
    known.extend(repo.entries()?);

    let mut trackers = HashMap::<Link, Tracker>::new();

    loop {
        let mut queue = LinkedList::from_iter(known.iter().cloned());
        while let Some(link) = queue.pop_front() {
            let result: Result<()> = try {
                let pad = Pad::fetch(&link).await?;

                let urls = pad.crawl()?;
                for url in urls {
                    if let Some(link) = Link::from_url(&config.crawl.servers, &url) {
                        if known.insert(link.clone()) {
                            queue.push_back(link);
                        }
                    }
                }

                let tracker = match trackers.entry(link.clone()) {
                    Entry::Occupied(occupied) => {
                        occupied.into_mut()
                            .update(&pad.content)
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(match repo.read(&link).await? {
                            Some(existing) => Tracker::from_existing(&existing, &pad.content),
                            None => Tracker::from_new(&pad.content),
                        })
                    }
                };

                if tracker.quiesce(config.notify.cool_down) {
                    let orig = repo.read(&link).await?;
                    repo.store(&link, &pad.content).await?;

                    notifier.notify(&pad, orig.as_deref()).await?
                }
            };

            if let Err(err) = result {
                eprintln!("Error processing link {}: {}", link, err);
                continue;
            }
        }

        tokio::time::sleep(config.crawl.interval).await;
    }
}
