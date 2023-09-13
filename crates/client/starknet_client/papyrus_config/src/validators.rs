//! Utils for config validations.

use validator::ValidationError;

/// Custom validation for ASCII string.
pub fn validate_ascii(name: &impl ToString) -> Result<(), ValidationError> {
    if !name.to_string().is_ascii() {
        return Err(ValidationError::new("ASCII Validation"));
    }
    Ok(())
}
