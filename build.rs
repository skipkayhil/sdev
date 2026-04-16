use std::env;

fn main() {
    let pwd = env::current_dir().expect("no pwd");
    let repo = gix::discover(pwd).expect("git repo");
    let sha = repo.head_commit().expect("HEAD commit").id;

    println!("cargo:rustc-env=SDEV_VCS_REVISION={}", sha);
}
