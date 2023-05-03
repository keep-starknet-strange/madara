use hex::FromHex;
use sp_core::{H256, U256};

const PREFIX: &str = "0x";

/// Removes the "0x" prefix from a given hexadecimal string
pub fn remove_prefix(input: &str) -> &str {
    input.strip_prefix(PREFIX).unwrap_or(input)
}

/// Converts a `H256` value to a hexadecimal string.
///
/// # Arguments
///
/// * `h256` - A `H256` value to convert to a hexadecimal string.
///
/// # Returns
///
/// A `Result` containing the hexadecimal string if the conversion was successful, or an error
/// message if the conversion failed.
pub fn h256_to_string(h256: H256) -> Result<String, String> {
    let bytes: [u8; 32] = h256.try_into().map_err(|e| format!("Failed to convert hex string to bytes: {:?}", e))?;
    let hex_string = hex::encode(bytes);
    let result =
        String::from_utf8(hex_string.into()).map_err(|e| format!("Failed to convert hex string to bytes: {:?}", e))?;
    Ok(PREFIX.to_string() + &result)
}

/// Converts a hexadecimal string to an H256 value, padding with zero bytes on the left if necessary
pub fn string_to_h256(hex_str: &str) -> Result<H256, String> {
    let hex_str = remove_prefix(hex_str);
    let mut padded_hex_str = hex_str.to_string();
    while padded_hex_str.len() < 64 {
        padded_hex_str.insert(0, '0');
    }
    let bytes =
        Vec::from_hex(&padded_hex_str).map_err(|e| format!("Failed to convert hex string to bytes: {:?}", e))?;
    Ok(H256::from_slice(&bytes))
}

/// Converts a `U256` value to a hexadecimal string with a width of 66 characters,
/// including the '0x' prefix.
///
/// # Arguments
///
/// * `u256` - A `U256` value to convert to a hexadecimal string.
///
/// # Returns
///
/// A `Result` containing the hexadecimal string if the conversion was successful, or an error
/// message if the conversion failed.
pub fn u256_to_string(u256: U256) -> Result<String, String> {
    Ok(format!("{:#066x}", u256))
}

/// Converts a u8 array of length 32 to a hexadecimal string with a width of 66 characters,
/// including the '0x' prefix.
///
/// # Arguments
///
/// * u8_array - A u8 array of length 32 to convert to a hexadecimal string.
///
/// # Returns
///
/// A Result containing the hexadecimal string if the conversion was successful, or an error message
/// if the conversion failed.
pub fn u8_array_to_string(arr: [u8; 32]) -> Result<String, String> {
    let hex_vec: Vec<String> = arr.iter().map(|byte| format!("{:02x}", byte)).collect();
    let hex_string = hex_vec.join("");
    Ok(format!("0x{}", hex_string))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{h256_to_string, string_to_h256, u256_to_string, u8_array_to_string, H256, U256};
    // Test case for the string_to_h256 function
    #[test]
    fn test_string_to_h256() {
        // Test case 1: Valid input with 64 characters
        let hex_str_1 = "0x0222882e457847df7ebaf981db2ff8ebb22c19d5b0a6a41dcc13cc2d775fbeb7";
        let expected_result_1 = H256::from_str(hex_str_1).unwrap();
        assert_eq!(string_to_h256(hex_str_1).unwrap(), expected_result_1);

        // Test case 2: Input with leading zeros
        let hex_str_2 = "0x0123456789abcdef";
        let expected_result_2 =
            H256::from_str("0x0000000000000000000000000000000000000000000000000123456789abcdef").unwrap();
        assert_eq!(string_to_h256(hex_str_2).unwrap(), expected_result_2);

        // Test case 3: Input with missing "0x" prefix
        let hex_str_3 = "222882e457847df7ebaf981db2ff8ebb22c19d5b0a6a41dcc13cc2d775fbeb7";
        let expected_result_3 =
            H256::from_str("0x0222882e457847df7ebaf981db2ff8ebb22c19d5b0a6a41dcc13cc2d775fbeb7").unwrap();
        assert_eq!(string_to_h256(hex_str_3).unwrap(), expected_result_3);

        // Test case 4: Input with invalid length
        let hex_str_4 = "0x222882e457847df7ebaf981db2ff8ebb22c19d5b0a6a41dcc13cc2d775fbeb7111111";
        assert!(string_to_h256(hex_str_4).is_err());
    }

	#[test]
    fn test_h256_to_string() {
        // Test case 1: Valid input with 64 characters
        let hex_str_1 = "0x0222882e457847df7ebaf981db2ff8ebb22c19d5b0a6a41dcc13cc2d775fbeb7";
        let input_1 = H256::from_str(hex_str_1).unwrap();
        assert_eq!(h256_to_string(input_1).unwrap(), hex_str_1);

        // Test case 2: Input with leading zeros
        let hex_str_2 = "0x0000000000000000000000000000000000000000000000000c13cc2d775fbeb7";
        let input_2 = H256::from_str(hex_str_2).unwrap();
        assert_eq!(h256_to_string(input_2).unwrap(), hex_str_2);
    }

    #[test]
    fn test_u256_to_string() {
        // Test with a U256 value of 0
        let u256_0 = U256::zero();
        assert_eq!(
            u256_to_string(u256_0),
            Ok("0x0000000000000000000000000000000000000000000000000000000000000000".to_owned())
        );

        // Test with a U256 value of 123456789
        let u256_123456789 = U256::from(123456789);
        assert_eq!(
            u256_to_string(u256_123456789),
            Ok("0x00000000000000000000000000000000000000000000000000000000075bcd15".to_owned())
        );
    }

    #[test]
    fn test_u8_array_to_hex_string() {
        // Test normal case
        let array = [255u8; 32];
        assert_eq!(
            u8_array_to_string(array).unwrap(),
            "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        );

        // Test input with leading zeros
        let array: [u8; 32] = [0; 32];
        assert_eq!(
            u8_array_to_string(array).unwrap(),
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
