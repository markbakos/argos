/// Security classification assigned by an application use case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionClassification {
    Read,
    Write,
    Privileged,
    Destructive,
}
