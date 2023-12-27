use crate::shell::ShellError;

pub mod git;
pub mod tmux;

type MetResult = Result<Status, Unmeetable>;
type MeetResult = Result<(), Unmeetable>;
type Reqs = Vec<Box<dyn Dep>>;

pub enum Status {
    Met,
    Unmet,
}

impl Status {
    fn is_met(&self) -> bool {
        matches!(self, Status::Met)
    }

    #[cfg(test)]
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
    #[error("dep is unmet after meet")]
    UnmetAfterMeet,
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

    fn process(&self) -> MeetResult {
        for req in self.reqs_to_met().iter() {
            req.process()?
        }

        if self.met()?.is_met() {
            return Ok(());
        }

        for req in self.reqs_to_meet().iter() {
            req.process()?
        }

        self.meet()?;

        match self.met()? {
            Status::Unmet => Err(Unmeetable::UnmetAfterMeet),
            Status::Met => Ok(()),
        }
    }
}

#[cfg(test)]
mod dep_tests {
    use super::*;

    const MET: MetResult = Ok(Status::Met);
    const UNMET: MetResult = Ok(Status::Unmet);

    struct BlankDep;
    impl Dep for BlankDep {
        fn met(&self) -> MetResult {
            MET
        }

        fn meet(&self) -> MeetResult {
            Ok(())
        }
    }

    #[test]
    fn processable() {
        assert!(BlankDep.process().is_ok());
    }

    struct RaisingMetDep;
    impl Dep for RaisingMetDep {
        fn met(&self) -> MetResult {
            // UnmetAfterMeet won't really be raised here, but it's easier to mock than creating a
            // nested ShellError and io::Error
            Err(Unmeetable::UnmetAfterMeet)
        }

        fn meet(&self) -> MeetResult {
            Ok(())
        }
    }

    #[test]
    fn handles_errors_in_met() {
        assert!(RaisingMetDep.process().is_err());
    }

    struct NeverMetDep;
    impl Dep for NeverMetDep {
        fn met(&self) -> MetResult {
            UNMET
        }

        fn meet(&self) -> MeetResult {
            Ok(())
        }
    }

    #[test]
    fn errors_when_unmet_after_meet() {
        assert!(NeverMetDep.process().is_err());
    }
}
