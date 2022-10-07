use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct Repo {
    owner: String,
    name: String,
}

impl Repo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn to_absolute_path(&self, root: &Path) -> PathBuf {
        root.join(&self.owner).join(&self.name)
    }
}

#[test]
fn test_to_absolute_path() {
    let repo = Repo {
        owner: "skipkayhil".to_string(),
        name: "dotfiles".to_string(),
    };
    assert_eq!(
        repo.to_absolute_path(Path::new("/home/hartley/src/github.com")),
        PathBuf::from("/home/hartley/src/github.com/skipkayhil/dotfiles")
    )
}

#[derive(Clone)]
pub struct MaybeOwnedRepo {
    owner: Option<String>,
    name: String,
}

impl MaybeOwnedRepo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owner(&self) -> &Option<String> {
        &self.owner
    }

    pub fn unwrap_or_else(
        &self,
        fallback: impl Fn(&str) -> Result<String, String>,
    ) -> Result<Repo, String> {
        let owner = match &self.owner {
            Some(owner) => owner.clone(),
            None => fallback(&self.name)?,
        };

        Ok(Repo {
            owner,
            name: self.name.clone(),
        })
    }
}

impl FromStr for MaybeOwnedRepo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();

        match parts[..] {
            [name] => Ok(Self {
                owner: None,
                name: name.to_string(),
            }),
            [owner, name] => Ok(Self {
                owner: Some(owner.to_string()),
                name: name.to_string(),
            }),
            _ => Err(format!("Invalid repo: {}", s)),
        }
    }
}

#[test]
fn parses_name_only() {
    let repo: MaybeOwnedRepo = "friday".parse().unwrap();
    assert_eq!(None, repo.owner);
    assert_eq!("friday", repo.name);
}

#[test]
fn parses_name_and_owner() {
    let repo: MaybeOwnedRepo = "rails/rails".parse().unwrap();
    assert_eq!("rails", repo.owner.unwrap());
    assert_eq!("rails", repo.name);
}
