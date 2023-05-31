use thiserror_no_std::Error;
/// Error that may occur while searching a Madara \[Log\] in the \[Digest\]
///
/// As for now only one single Madara \[Log\] is expected per \[Digest\].
/// No more, no less.
#[derive(Clone, Debug, Error)]
pub enum FindLogError {
    /// There was no Madara \[Log\] in the \[Digest\]
    #[error("Madara log not found")]
    NotLog,
    /// There was multiple Madara \[Log\] in the \[Digest\]
    #[error("Multiple Madara logs found")]
    MultipleLogs,
}
