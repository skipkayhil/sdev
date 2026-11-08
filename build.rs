use std::env;

fn main() {
    if let Ok(vcs_revision) = env::var("SDEV_VCS_REVISION") {
        println!("cargo:rustc-env=SDEV_VCS_REVISION={}", vcs_revision);
    } else {
        let pwd = env::current_dir().expect("no pwd");
        let repo = gix::discover(pwd).expect("git repo");
        let vcs_revision = repo.head_commit().expect("HEAD commit").id;
        println!("cargo:rustc-env=SDEV_VCS_REVISION={}", vcs_revision);
    };
}
