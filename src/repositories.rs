pub mod git_repos {
    use std::collections::VecDeque;
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

    pub trait Repository {
        fn fetch_all(&self) -> Result<Vec<GitRepo>, FetchAllError>;
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
        fn fetch_all(&self) -> Result<Vec<GitRepo>, FetchAllError> {
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
    }
}
