pub mod pr {
    use gix;
    use gix::remote::Direction;
    use std::env;

    #[derive(thiserror::Error, Debug)]
    #[error("error opening pr")]
    enum Error {
        DetachedHead,
        MissingRemoteHost,
        MissingRemoteUrl,
        MissingTargetRemote,
        PathEncoding(#[source] std::string::FromUtf8Error),
    }

    enum Remote {
        Origin,
        Upstream,
    }

    enum UrlStrategy {
        GithubOrigin(String),
        GithubUpstream(String),
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
                .ok_or(Error::MissingRemoteUrl)?;
            let host = url.host().ok_or(Error::MissingRemoteHost)?;

            let path = {
                let utf = url.path.strip_suffix(b".git").unwrap_or(&url.path).to_vec();
                String::from_utf8(utf).map_err(Error::PathEncoding)?
            };

            let url_string = format!("{}/{}", host, path);

            Ok(match host {
                "github.com" => match remote_type {
                    Remote::Origin => Self::GithubOrigin(url_string),
                    Remote::Upstream => Self::GithubUpstream(url_string),
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
                // Self::GithubUpstream(u)
                _ => todo!(),
            }
        }
    }

    pub fn run(target: &Option<String>) -> anyhow::Result<()> {
        let pwd = env::current_dir()?;
        let repo = gix::discover(pwd)?;

        let head = repo.head_ref()?.ok_or(Error::DetachedHead)?;
        let branch = head.name().file_name();

        let strategy = UrlStrategy::try_from(&repo)?;

        // TODO: support not GitHub URL format
        println!("{}", strategy.to_url(branch, target));

        Ok(())
    }
}
