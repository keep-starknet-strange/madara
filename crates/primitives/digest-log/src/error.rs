/// Error that may occur while searching a Madara [Log] in the [Digest]
///
/// As for now only one single Madara [Log] is expected per [Digest].
/// No more, no less.
#[derive(Clone, Debug)]
pub enum FindLogError {
    /// There was no Madara [Log] in the [Digest]
    NotLog,
    /// There was multiple Madara [Log] in the [Digest]
    MultipleLogs,
}

impl core::fmt::Display for FindLogError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FindLogError::NotLog => write!(f, "Madara log not found"),
            FindLogError::MultipleLogs => write!(f, "Multiple Madara logs found"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FindLogError {}
