mod clone;
pub mod tmux;

pub use clone::Clone;

pub trait Dep {
    fn met(&self) -> bool;
    fn meet(&self) -> bool;

    fn process(&self) -> bool {
        if self.met() {
            return true;
        }

        self.meet();

        self.met()
    }
}
