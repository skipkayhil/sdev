use bstr::ByteSlice;
use gix::url::{Scheme, Url};

use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

#[derive(Clone)]
pub struct GitRepo {
    name: String,
    path: PathBuf,
}

impl GitRepo {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn relative_path(&self, root: &Path) -> &Path {
        self.path.strip_prefix(root).unwrap_or(&self.path)
    }
}

#[derive(Clone)]
pub enum GitRepoSource {
    Name(String),
    Path(String),
    Url {
        url: Url,
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
                    url: url.clone(),
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
            matches!(repo, GitRepoSource::Url { url, .. } if url.to_string() == "https://github.com/skipkayhil/sdev".to_string())
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
