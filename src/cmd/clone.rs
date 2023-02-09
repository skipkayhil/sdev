use std::path::PathBuf;
use std::process::Command;

use crate::cmd::PrintableCommand;
use crate::repo::GitRepoSource;
use crate::Config;

pub fn run(source: &GitRepoSource, config: Config) -> Result<(), String> {
    let (url, path) = url_and_path_for_source(source, config);

    shell!("git", "clone", url, path)
        .print_and_run()
        .map_err(From::from)
}

fn url_and_path_for_source(source: &GitRepoSource, config: Config) -> (String, PathBuf) {
    match source {
        GitRepoSource::Name(s) => (
            format!("git@{}:{}/{s}.git", config.host, config.user),
            config.root.join(config.host).join(config.user).join(s),
        ),
        GitRepoSource::Path(s) => (
            format!("git@{}:{s}.git", config.host),
            config.root.join(config.host).join(s),
        ),
        GitRepoSource::Url(u) => (
            u.as_str().to_string(),
            // TODO: don't unwrap the host
            config
                .root
                .join(u.host_str().unwrap())
                .join(u.path().trim_start_matches('/').trim_end_matches(".git")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn url_and_path_use_host_and_user_when_given_name() {
        let source = GitRepoSource::Name("sdev".to_string());
        let (url, path) = url_and_path_for_source(&source, mock_config());

        assert_eq!("git@github.com:skipkayhil/sdev.git", url);
        assert_eq!(
            PathBuf::from("/home/skipkayhil/src/github.com/skipkayhil/sdev"),
            path
        );
    }

    #[test]
    fn url_and_path_use_host_when_given_path() {
        let source = GitRepoSource::Path("ruby/ruby".to_string());
        let (url, path) = url_and_path_for_source(&source, mock_config());

        assert_eq!("git@github.com:ruby/ruby.git", url);
        assert_eq!(
            PathBuf::from("/home/skipkayhil/src/github.com/ruby/ruby"),
            path
        );
    }

    #[test]
    fn url_and_path_are_constructed_when_given_url() {
        let url = Url::parse("https://aur.archlinux.org/google-chrome.git").unwrap();
        let source = GitRepoSource::Url(url);
        let (url, path) = url_and_path_for_source(&source, mock_config());

        assert_eq!("https://aur.archlinux.org/google-chrome.git", url);
        assert_eq!(
            PathBuf::from("/home/skipkayhil/src/aur.archlinux.org/google-chrome"),
            path
        );
    }

    fn mock_config() -> Config {
        Config {
            host: "github.com".to_string(),
            root: PathBuf::from("/home/skipkayhil/src"),
            user: "skipkayhil".to_string(),
        }
    }
}
