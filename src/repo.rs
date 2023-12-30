use bstr::ByteSlice;
use gix::url::{Scheme, Url};

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

#[derive(Clone)]
enum GitHost {
    Github,
    Other,
}

impl From<&OsStr> for GitHost {
    fn from(value: &OsStr) -> Self {
        if value == "github.com" {
            Self::Github
        } else {
            Self::Other
        }
    }
}

#[derive(Clone)]
pub struct GitRepo {
    host: GitHost,
    name: String,
    path: PathBuf,
}

pub enum TryFromAbsoluteError {
    InvalidDir,
    NotInRoot,
    TryFromFsError(TryFromFsError),
}

pub enum TryFromFsError {
    Encoding,
    NotARepo(PathBuf),
}

impl GitRepo {
    pub fn try_from_absolute(
        path: PathBuf,
        root: &PathBuf,
    ) -> Result<GitRepo, TryFromAbsoluteError> {
        let host = {
            let Ok(relative_path) = path.strip_prefix(root) else {
                return Err(TryFromAbsoluteError::NotInRoot);
            };

            let maybe_host = relative_path.components().find_map(|c| match c {
                Component::Normal(segment) => Some(segment),
                _ => None,
            });

            let Some(host) = maybe_host else {
                return Err(TryFromAbsoluteError::InvalidDir);
            };

            host
        };

        let Some(name) = path.file_name() else {
            return Err(TryFromAbsoluteError::InvalidDir);
        };

        Self::try_from_fs(name, path.clone(), host).map_err(TryFromAbsoluteError::TryFromFsError)
    }

    pub fn try_from_fs(
        raw_name: &OsStr,
        path: PathBuf,
        host_domain: &OsStr,
    ) -> Result<Self, TryFromFsError> {
        if Path::new(&path).join(".git").read_dir().is_err() {
            return Err(TryFromFsError::NotARepo(path));
        }

        let Some(name) = raw_name.to_str() else {
            return Err(TryFromFsError::Encoding);
        };

        Ok(Self {
            name: name.into(),
            path,
            host: host_domain.into(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
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
