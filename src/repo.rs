use std::str::FromStr;

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
