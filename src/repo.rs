use bstr::ByteSlice;
use gix_url::{Scheme, Url};

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
    Url {
        url: String,
        host: String,
        path: PathBuf,
    },
}

impl FromStr for GitRepoSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::try_from(s).map_err(|e| e.to_string())?;

        let path = url.path.to_path().map_err(|e| e.to_string())?;

        match url.scheme {
            Scheme::File => {
                for component in path.components() {
                    let Component::Normal(..) = component else {
                        return Err(format!("invalid repo: {s}"));
                    };
                }

                if s.contains('/') {
                    Ok(GitRepoSource::Path(s.into()))
                } else {
                    Ok(GitRepoSource::Name(s.into()))
                }
            }
            _ => {
                let host = url.host().ok_or("invalid host for: {s}")?;
                let relative_path = {
                    let mut buffer = PathBuf::new();

                    for component in path.with_extension("").components() {
                        match component {
                            Component::RootDir => (),
                            Component::Normal(..) => buffer.push(component),
                            _ => return Err(format!("invalid repo: {s}")),
                        };
                    }

                    buffer
                };

                Ok(GitRepoSource::Url {
                    url: s.to_string(),
                    host: host.to_string(),
                    path: relative_path,
                })
            }
        }
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
            matches!(repo, GitRepoSource::Url { url, .. } if url.as_str() == "https://github.com/skipkayhil/sdev".to_string())
        );
    }

    #[test]
    fn errors_on_url_path_traversal() {
        let result = "git@github.com:../evil.git".parse::<GitRepoSource>();

        assert!(result.is_err());
        assert_eq!(
            "invalid repo: git@github.com:../evil.git",
            result.err().unwrap()
        );
    }
}
