pub mod git_repos {
    use std::path::PathBuf;

    use jwalk::WalkDirGeneric;

    use crate::repo::GitRepo;

    pub trait Repository {
        fn fetch_all(&self) -> Vec<GitRepo>;
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
        fn fetch_all(&self) -> Vec<GitRepo> {
            let mut repos = Vec::new();

            let walk_dir = WalkDirGeneric::<((), bool)>::new(&self.root).process_read_dir(
                |_depth, _path, _read_dir_state, children| {
                    for dir_entry in children.iter_mut().flatten() {
                        if dir_entry.path().join(".git").read_dir().is_ok() {
                            dir_entry.read_children_path = None;
                            dir_entry.client_state = true;
                        }
                    }
                },
            );

            for dir_entry in walk_dir.into_iter().flatten() {
                if !dir_entry.client_state {
                    continue;
                };

                if let Some(name) = dir_entry.file_name.to_str() {
                    repos.push(GitRepo::new(name.into(), dir_entry.path()))
                }
            }

            repos
        }
    }
}
