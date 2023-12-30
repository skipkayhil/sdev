pub mod git_repos {
    use std::collections::{HashMap, VecDeque};
    use std::io;
    use std::path::{Path, PathBuf};

    use crate::repo::{GitRepo, TryFromAbsoluteError, TryFromFsError};

    #[derive(thiserror::Error, Debug)]
    pub enum FetchAllError {
        #[error("error reading root src dir")]
        ReadRoot(#[source] io::Error),
    }

    #[derive(thiserror::Error, Debug)]
    pub enum FetchOneError {
        #[error("unknown git repo: {path}")]
        UnknownRepo {
            path: PathBuf,
            #[source]
            source: TryFromAbsoluteError,
        },
    }

    pub trait Repository {
        fn fetch_all(&mut self) -> Result<Vec<GitRepo>, FetchAllError>;
        fn fetch_one(&mut self, path: &Path) -> Result<GitRepo, FetchOneError>;
    }

    pub struct FileSystemRepository {
        root: PathBuf,
    }

    impl FileSystemRepository {
        pub fn new(root: PathBuf) -> Self {
            Self { root }
        }
    }

    impl Repository for FileSystemRepository {
        fn fetch_all(&mut self) -> Result<Vec<GitRepo>, FetchAllError> {
            let host_entries = self
                .root
                .read_dir()
                .map_err(FetchAllError::ReadRoot)?
                .filter_map(|dir| dir.ok());

            let mut queue = VecDeque::new();
            let mut repos = Vec::new();

            for host_entry in host_entries {
                let host = host_entry.file_name();

                if let Ok(repo_iter) = host_entry.path().read_dir() {
                    queue.extend(repo_iter.filter_map(|dir| dir.ok()))
                }

                while let Some(dir_entry) = queue.pop_front() {
                    let name = dir_entry.file_name();
                    let path = dir_entry.path();

                    match GitRepo::try_from_fs(&name, &path, &host) {
                        Ok(repo) => repos.push(repo),
                        Err(TryFromFsError::NotARepo(folder)) => {
                            if let Ok(dir_iter) = folder.read_dir() {
                                queue.extend(dir_iter.filter_map(|dir| dir.ok()));
                            }
                        }
                        _ => (),
                    }
                }
            }

            Ok(repos)
        }

        fn fetch_one(&mut self, path: &Path) -> Result<GitRepo, FetchOneError> {
            GitRepo::try_from_absolute(path, &self.root).map_err(|e| FetchOneError::UnknownRepo {
                source: e,
                path: path.to_owned(),
            })
        }
    }

    pub struct CachingRepository<T: Repository> {
        repository: T,
        cache: HashMap<PathBuf, GitRepo>,
    }

    impl<T> CachingRepository<T>
    where
        T: Repository,
    {
        pub fn new(repository: T) -> Self {
            Self {
                repository,
                cache: HashMap::new(),
            }
        }

        pub fn fetch_one_from_cache(&self, path: &Path) -> Option<GitRepo> {
            self.cache.get(path).cloned()
        }
    }

    impl<T> Repository for CachingRepository<T>
    where
        T: Repository,
    {
        fn fetch_all(&mut self) -> Result<Vec<GitRepo>, FetchAllError> {
            if self.cache.is_empty() {
                let all = self.repository.fetch_all()?;

                for git_repo in all.into_iter() {
                    self.cache.insert(git_repo.path().into(), git_repo);
                }
            }

            Ok(self.cache.values().cloned().collect())
        }

        fn fetch_one(&mut self, path: &Path) -> Result<GitRepo, FetchOneError> {
            if let Some(repo) = self.cache.get(path) {
                return Ok(repo.clone());
            }

            let repo = self.repository.fetch_one(path)?;
            self.cache.insert(repo.path().into(), repo.clone());

            Ok(repo)
        }
    }
}
