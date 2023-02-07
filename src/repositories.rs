pub mod git_repos {
    use std::collections::{HashMap, VecDeque};
    use std::io;
    use std::path::PathBuf;

    use crate::repo::GitRepo;

    pub enum FetchAllError {
        IoError(io::Error),
    }

    impl From<FetchAllError> for String {
        fn from(value: FetchAllError) -> Self {
            match value {
                FetchAllError::IoError(e) => format!("error fetching git repos: {e}"),
            }
        }
    }

    impl From<io::Error> for FetchAllError {
        fn from(e: io::Error) -> Self {
            FetchAllError::IoError(e)
        }
    }

    #[derive(Debug)]
    pub enum FetchOneError {
        UnknownRepo(PathBuf),
        IoError(io::Error),
    }

    impl From<FetchOneError> for String {
        fn from(value: FetchOneError) -> Self {
            match value {
                FetchOneError::UnknownRepo(p) => format!("unknown git repo: {}", p.display()),
                FetchOneError::IoError(e) => format!("error fetching git repo: {e}"),
            }
        }
    }

    impl From<io::Error> for FetchOneError {
        fn from(e: io::Error) -> Self {
            FetchOneError::IoError(e)
        }
    }
    pub trait Repository {
        fn fetch_all(&mut self) -> Result<Vec<GitRepo>, FetchAllError>;
        fn fetch_one(&mut self, path: String) -> Result<GitRepo, FetchOneError>;
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
            let root_entries = self.root.read_dir()?.filter_map(|dir| dir.ok());

            let mut queue = VecDeque::from_iter(root_entries);
            let mut repos = Vec::new();

            while let Some(dir_entry) = queue.pop_front() {
                let path = dir_entry.path();

                match GitRepo::try_from(path) {
                    Ok(repo) => repos.push(repo),
                    Err(err) => {
                        if let Ok(dir_iter) = err.read_dir() {
                            queue.extend(dir_iter.filter_map(|dir| dir.ok()));
                        }
                    }
                }
            }

            Ok(repos)
        }

        fn fetch_one(&mut self, path: String) -> Result<GitRepo, FetchOneError> {
            GitRepo::try_from(PathBuf::from(path)).map_err(FetchOneError::UnknownRepo)
        }
    }

    pub struct CachingRepository<T: Repository> {
        repository: T,
        cache: HashMap<String, GitRepo>,
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

        pub fn fetch_one_from_cache(&self, path: &str) -> Option<GitRepo> {
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

        fn fetch_one(&mut self, path: String) -> Result<GitRepo, FetchOneError> {
            if let Some(repo) = self.cache.get(&path) {
                return Ok(repo.clone());
            }

            let repo = self.repository.fetch_one(path)?;
            self.cache.insert(repo.path().into(), repo.clone());

            Ok(repo)
        }
    }
}
