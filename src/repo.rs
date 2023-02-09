use url::Url;

use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

#[derive(Clone)]
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
pub enum GitRepoSource {
    Name(String),
    Path(String),
    Url(Url),
}

impl FromStr for GitRepoSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(url) = Url::parse(s) {
            return Ok(GitRepoSource::Url(url));
        }

        for component in Path::new(s).components() {
            let Component::Normal(..) = component else {
                return Err(format!("invalid repo: {s}"));
            };
        }

        let source = if s.contains('/') {
            GitRepoSource::Path(s.to_string())
        } else {
            GitRepoSource::Name(s.to_string())
        };

        Ok(source)
    }
}

#[cfg(test)]
mod git_repo_source_tests {
    use super::GitRepoSource;

    #[test]
    fn parse_name() {
        let repo: GitRepoSource = "friday".parse().unwrap();

        assert!(matches!(repo, GitRepoSource::Name(r) if r == "friday".to_string()));
    }

    #[test]
    fn parse_short_path() {
        let repo: GitRepoSource = "rails/rails".parse().unwrap();

        assert!(matches!(repo, GitRepoSource::Path(r) if r == "rails/rails".to_string()));
    }

    #[test]
    fn errors_on_absolute_path() {
        let result = "/opt".parse::<GitRepoSource>();

        assert!(result.is_err());
        assert_eq!("invalid repo: /opt", result.err().unwrap());
    }

    #[test]
    fn errors_on_path_traversal() {
        let result = "../evil".parse::<GitRepoSource>();

        assert!(result.is_err());
        assert_eq!("invalid repo: ../evil", result.err().unwrap());
    }

    #[test]
    fn parse_http_url() {
        let repo: GitRepoSource = "https://github.com/skipkayhil/sdev".parse().unwrap();

        assert!(
            matches!(repo, GitRepoSource::Url(u) if u.as_str() == "https://github.com/skipkayhil/sdev".to_string())
        );
    }
}
