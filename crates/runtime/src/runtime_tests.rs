//! Tests for the runtime.

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use sp_core::hexdisplay::HexDisplay;

    use crate::sp_api_hidden_includes_construct_runtime::hidden_include::traits::WhitelistedStorageKeys;
    use crate::*;
    #[test]
    fn check_whitelist() {
        let whitelist: HashSet<String> = AllPalletsWithSystem::whitelisted_storage_keys()
            .iter()
            .map(|e| HexDisplay::from(&e.key).to_string())
            .collect();

        // Block Number
        assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac"));
        // Execution Phase
        assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a"));
        // Event Count
        assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850"));
        // System Events
        assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7"));
    }
}
