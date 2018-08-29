#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct InspectSecret<'a, T: 'a>(pub &'a T);

