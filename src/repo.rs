use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct GitRepo {
    path: String,
    name: String,
}

impl GitRepo {
    fn new(path: String, name: String) -> Self {
        Self { path, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl TryFrom<PathBuf> for GitRepo {
    type Error = PathBuf;

    fn try_from(value: PathBuf) -> Result<Self, PathBuf> {
        let Some(raw_name) = value.file_name() else {
            return Err(value);
        };
        let name = match raw_name.to_str() {
            Some(name) => name.to_string(),
            None => return Err(value),
        };

        let path_as_os_string = value.into_os_string();

        let path = match path_as_os_string.into_string() {
            Ok(path) => path,
            Err(os_str) => return Err(os_str.into()),
        };

        if Path::new(&path).join(".git").read_dir().is_err() {
            return Err(path.into());
        }

        Ok(Self::new(path, name))
    }
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
            _ => Err(format!("Invalid repo: {s}")),
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
