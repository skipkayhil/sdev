pub mod pr {
    use gix;
    use gix::remote::Direction;
    use std::env;

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
            let (remote, remote_type) = if let Ok(remote) = repo.find_remote("upstream") {
                (remote, Remote::Upstream)
            } else if let Ok(remote) = repo.find_remote("origin") {
                (remote, Remote::Origin)
            } else {
                Err(Error::MissingTargetRemote)?
            };

            let url = remote
                .url(Direction::Fetch)
                .ok_or_else(|| Error::MissingRemoteUrl((&remote_type).into()))?;
            let host = url.host().ok_or(Error::MissingRemoteHost)?;

            let path = {
                let utf = url.path.strip_suffix(b".git").unwrap_or(&url.path).to_vec();
                String::from_utf8(utf).map_err(Error::PathEncoding)?
            };

            let url_string = format!("{}/{}", host, path);

            Ok(match host {
                "github.com" => match remote_type {
                    Remote::Origin => Self::GithubOrigin(url_string),
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
                            url: url_string,
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

        println!("Opening {url}");

        opener::open(url)?;

        Ok(())
    }
}
