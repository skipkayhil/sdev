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

    pub fn to_path_with_base(&self, base: &str) -> PathBuf {
        Path::new(base)
            .join("src")
            .join("github.com")
            .join(&self.owner)
            .join(&self.name)
    }

    pub fn to_url(&self) -> String {
        format!("git@github.com:{}/{}.git", &self.owner, &self.name)
    }
}

#[test]
fn test_to_path_with_base() {
    let repo = Repo {
        owner: "skipkayhil".to_string(),
        name: "dotfiles".to_string(),
    };
    assert_eq!(
        repo.to_path_with_base("/home/hartley"),
        PathBuf::from("/home/hartley/src/github.com/skipkayhil/dotfiles")
    )
}

#[test]
fn test_to_url() {
    let repo = Repo {
        owner: "skipkayhil".to_string(),
        name: "dotfiles".to_string(),
    };
    assert_eq!(repo.to_url(), "git@github.com:skipkayhil/dotfiles.git")
}

impl FromStr for Repo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();

        match parts[..] {
            [name] => Ok(Self {
                owner: "skipkayhil".to_string(),
                name: name.to_string(),
            }),
            [owner, name] => Ok(Self {
                owner: owner.to_string(),
                name: name.to_string(),
            }),
            _ => Err(format!("Invalid repo: {}", s)),
        }
    }
}

#[test]
fn parses_name_only() {
    let repo: Repo = "friday".parse().unwrap();
    assert_eq!("skipkayhil", repo.owner);
    assert_eq!("friday", repo.name);
}

#[test]
fn parses_name_and_owner() {
    let repo: Repo = "rails/rails".parse().unwrap();
    assert_eq!("rails", repo.owner);
    assert_eq!("rails", repo.name);
}
