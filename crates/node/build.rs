use std::env;

use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
    // Check if the feature flag is enabled
    let feature_enabled = env::var("CARGO_FEATURE_DEV").is_ok();
    // Check if we are in release mode
    let debug_mode = env::var("PROFILE").map(|p| p == "debug").unwrap_or(false);

    if feature_enabled && !debug_mode {
        // Emit a compile error if the feature is enabled in release mode
        panic!("The feature 'dev' can only be enabled in debug mode.");
    }

    generate_cargo_keys();

    rerun_if_git_head_changed();
}
