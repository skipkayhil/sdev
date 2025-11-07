pub mod pr {
    use gix;
    use gix::remote::Direction;
    use std::env;

    use crate::shell;

    const ORIGIN: &str = "origin";
    const UPSTREAM: &str = "upstream";

    #[derive(thiserror::Error, Debug)]
    #[error("error opening pr")]
    enum Error {
        DetachedHead,
        MissingOriginForFork,
        MissingRemoteHost,
        MissingRemoteUrl(&'static str),
        MissingTargetRemote,
        PathEncoding(#[source] std::string::FromUtf8Error),
    }

    enum Remote {
        Origin,
        Upstream,
    }

    impl From<&Remote> for &str {
        fn from(r: &Remote) -> Self {
            match r {
                Remote::Origin => ORIGIN,
                Remote::Upstream => UPSTREAM,
            }
        }
    }

    enum UrlStrategy {
        GithubOrigin {
            host: String,
            path: String,
        },
        GithubUpstream {
            host: String,
            path: String,
            source: String,
        },
        Unknown,
    }

    impl TryFrom<&gix::Repository> for UrlStrategy {
        type Error = Error;

        fn try_from(repo: &gix::Repository) -> Result<Self, Error> {
            let (target_remote, remote_type) = if let Ok(upstream) = repo.find_remote(UPSTREAM) {
                (upstream, Remote::Upstream)
            } else if let Ok(origin) = repo.find_remote(ORIGIN) {
                (origin, Remote::Origin)
            } else {
                Err(Error::MissingTargetRemote)?
            };

            let target_git_url = target_remote
                .url(Direction::Fetch)
                .ok_or_else(|| Error::MissingRemoteUrl((&remote_type).into()))?;
            let target_host = target_git_url.host().ok_or(Error::MissingRemoteHost)?;

            let target_path = Self::normalized_path(target_git_url)?;

            Ok(match target_host {
                "github.com" => match remote_type {
                    Remote::Origin => Self::GithubOrigin {
                        host: target_host.into(),
                        path: target_path,
                    },
                    Remote::Upstream => {
                        let origin = repo
                            .find_remote(ORIGIN)
                            .map_err(|_| Error::MissingOriginForFork)?;

                        let url = origin
                            .url(Direction::Fetch)
                            .ok_or(Error::MissingRemoteUrl(ORIGIN))?;

                        let source = {
                            let utf = url.path.strip_suffix(b".git").unwrap_or(&url.path).to_vec();
                            let path = String::from_utf8(utf).map_err(Error::PathEncoding)?;
                            path.split('/')
                                .next()
                                .expect("remote path is missing a /")
                                .to_string()
                        };

                        Self::GithubUpstream {
                            host: target_host.into(),
                            path: target_path,
                            source,
                        }
                    }
                },
                _ => Self::Unknown,
            })
        }
    }

    impl UrlStrategy {
        fn to_url(&self, branch: &bstr::BStr, target: &Option<String>) -> String {
            match self {
                Self::GithubOrigin { host, path } => {
                    let target_string = target
                        .as_ref()
                        .map(|name| format!("{name}..."))
                        .unwrap_or_default();

                    format!("https://{host}{path}/pull/{target_string}{branch}")
                }
                Self::GithubUpstream { host, path, source } => {
                    let target_string = target
                        .as_ref()
                        .map(|name| format!("{name}..."))
                        .unwrap_or_default();

                    format!("https://{host}{path}/pull/{target_string}{source}:{branch}")
                }
                _ => todo!(),
            }
        }

        fn normalized_path(git_url: &gix::Url) -> Result<String, Error> {
            let utf = git_url
                .path
                .strip_suffix(b".git")
                .unwrap_or(&git_url.path)
                .to_vec();
            let path = String::from_utf8(utf).map_err(Error::PathEncoding)?;

            Ok(match git_url.scheme {
                // SSH Scheme has a : between host/path, so no leading /
                gix::url::Scheme::Ssh => format!("/{path}"),
                // All? other Schemes have / between host/path, so path has a leading /
                _ => path,
            })
        }
    }

    pub fn run(target: &Option<String>) -> anyhow::Result<()> {
        let pwd = env::current_dir()?;
        let repo = gix::discover(pwd)?;

        let head = repo.head_ref()?.ok_or(Error::DetachedHead)?;
        let branch = head.name().file_name();

        let url = UrlStrategy::try_from(&repo)?.to_url(branch, target);

        if let Some(remote) = head.remote(Direction::Fetch) {
            // TODO: it would be cool if a "git status" type check could be added here which ensure
            // the local branch is up to date with the remote branch
            remote?.name().expect("remote name not persisted?");
        } else {
            shell::new!("git", "push", "--set-upstream", ORIGIN, &branch.to_string()).run(true)?;
        }

        println!("Opening {url}");

        opener::open(url)?;

        Ok(())
    }
}
