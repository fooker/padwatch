use std::path::PathBuf;

use anyhow::Result;

use crate::Link;

pub struct Repo {
    path: PathBuf,
}

impl Repo {
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        tokio::fs::create_dir_all(&path).await?;

        return Ok(Self {
            path,
        });
    }

    pub async fn read(&self, link: &Link) -> Result<Option<String>> {
        let path = self.path_for(link);

        if tokio::fs::try_exists(&path).await? {
            return Ok(Some(tokio::fs::read_to_string(&path).await?));
        }

        return Ok(None);
    }

    pub async fn store(&mut self, link: &Link, content: &String) -> Result<()> {
        let path = self.path_for(link);

        // Just ensure server directory exists
        tokio::fs::create_dir_all(&path.parent().expect("pad path has parent")).await?;

        tokio::fs::write(path, content).await?;

        return Ok(());
    }

    pub fn entries(&self) -> Result<Vec<Link>> {
        let mut links = Vec::new();

        for server in std::fs::read_dir(&self.path)? {
            let server = server?.file_name();

            for name in std::fs::read_dir(&self.path.join(&server))? {
                let name = name?.file_name();

                links.push(Link::new(
                    server.to_string_lossy().to_string(),
                    name.to_string_lossy().to_string(),
                ));
            }
        }

        return Ok(links);
    }

    fn path_for(&self, link: &Link) -> PathBuf {
        return self.path
            .join(&link.server)
            .join(&link.name);
    }
}