mod clone;
pub mod tmux;

pub use clone::Clone;

pub trait Dep {
    fn met(&self) -> bool;
    fn meet(&self) -> bool;

    fn reqs_to_met(&self) -> Vec<Box<dyn Dep>> {
        vec![]
    }

    fn reqs_to_meet(&self) -> Vec<Box<dyn Dep>> {
        vec![]
    }

    fn process(&self) -> bool {
        for req in self.reqs_to_met().iter() {
            if !req.process() {
                return false;
            }
        }

        if self.met() {
            return true;
        }

        for req in self.reqs_to_meet().iter() {
            if !req.process() {
                return false;
            }
        }

        self.meet();

        self.met()
    }
}
