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
        MissingRemoteUrl(String),
        MissingTargetRemote,
        PathEncoding(#[source] std::string::FromUtf8Error),
    }

    enum Remote {
        Origin,
        Upstream,
    }

    impl From<&Remote> for String {
        fn from(r: &Remote) -> Self {
            match r {
                Remote::Origin => ORIGIN.to_string(),
                Remote::Upstream => UPSTREAM.to_string(),
            }
        }
    }

    enum UrlStrategy {
        GithubOrigin(String),
        GithubUpstream { url: String, origin: String },
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

            let request_url = {
                let utf = target_git_url
                    .path
                    .strip_suffix(b".git")
                    .unwrap_or(&target_git_url.path)
                    .to_vec();
                let path = String::from_utf8(utf).map_err(Error::PathEncoding)?;

                format!("{}/{}", target_host, path)
            };

            Ok(match target_host {
                "github.com" => match remote_type {
                    Remote::Origin => Self::GithubOrigin(request_url),
                    Remote::Upstream => {
                        let Ok(origin) = repo.find_remote(ORIGIN) else {
                            Err(Error::MissingOriginForFork)?
                        };

                        let url = origin
                            .url(Direction::Fetch)
                            .ok_or_else(|| Error::MissingRemoteUrl(ORIGIN.to_string()))?;

                        let origin = {
                            let utf = url.path.strip_suffix(b".git").unwrap_or(&url.path).to_vec();
                            let path = String::from_utf8(utf).map_err(Error::PathEncoding)?;
                            path.split('/')
                                .next()
                                .expect("remote path is missing a /")
                                .to_string()
                        };

                        Self::GithubUpstream {
                            url: request_url,
                            origin,
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
                Self::GithubOrigin(u) => {
                    let target_string = target
                        .as_ref()
                        .map_or("".to_string(), |name| format!("{}...", name));

                    format!("https://{}/pull/{}{}", u, target_string, branch)
                }
                Self::GithubUpstream { url, origin } => {
                    let target_string = target
                        .as_ref()
                        .map_or("".to_string(), |name| format!("{}...", name));

                    format!(
                        "https://{}/pull/{}{}:{}",
                        url, target_string, origin, branch
                    )
                }
                _ => todo!(),
            }
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
