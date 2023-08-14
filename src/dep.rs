use crate::shell::ShellError;

mod clone;
pub mod tmux;

pub use clone::Clone;

type MetResult = Result<Status, Unmeetable>;
type MeetResult = Result<(), Unmeetable>;
type Reqs = Vec<Box<dyn Dep>>;

const MET: MetResult = Ok(Status::Met);
const UNMET: MetResult = Ok(Status::Unmet);

pub enum Status {
    Met,
    Unmet,
}

impl Status {
    fn is_met(&self) -> bool {
        matches!(self, Status::Met)
    }

    fn is_unmet(&self) -> bool {
        matches!(self, Status::Unmet)
    }
}

impl From<bool> for Status {
    fn from(b: bool) -> Self {
        if b {
            Status::Met
        } else {
            Status::Unmet
        }
    }
}

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn from_bool() {
        assert!(matches!(Status::from(true), Status::Met));
        assert!(matches!(Status::from(false), Status::Unmet));
    }

    #[test]
    fn is_met() {
        assert!(Status::Met.is_met());
        assert!(!Status::Unmet.is_met());
    }

    #[test]
    fn is_unmet() {
        assert!(!Status::Met.is_unmet());
        assert!(Status::Unmet.is_unmet());
    }
}

#[derive(thiserror::Error, Debug)]
#[error("unmeetable")]
pub enum Unmeetable {
    Shell(#[from] ShellError),
}

pub trait Dep {
    fn met(&self) -> MetResult;
    fn meet(&self) -> MeetResult;

    fn reqs_to_met(&self) -> Reqs {
        vec![]
    }

    fn reqs_to_meet(&self) -> Reqs {
        vec![]
    }

    fn process(&self) -> MetResult {
        for req in self.reqs_to_met().iter() {
            if req.process()?.is_unmet() {
                return UNMET;
            }
        }

        if self.met()?.is_met() {
            return MET;
        }

        for req in self.reqs_to_meet().iter() {
            if req.process()?.is_unmet() {
                return UNMET;
            }
        }

        self.meet()?;

        self.met()
    }
}
