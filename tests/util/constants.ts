export const BASE_PATH = process.env.BASE_PATH;
export const CUSTOM_SPEC_PATH = process.env.CUSTOM_SPEC_PATH;

export const DISPLAY_LOG = process.env.MADARA_LOG || false;
export const MADARA_LOG = process.env.MADARA_LOG || "info";
export const DEBUG_MODE = process.env.DEBUG_MODE || false;

export const BINARY_PATH =
  process.env.BINARY_PATH || `../target/release/madara`;

// Is undefined by default as the path is dependent of the runtime.
export const OVERRIDE_RUNTIME_PATH =
  process.env["OVERRIDE_RUNTIME_PATH"] || undefined;
export const SPAWNING_TIME = 500000;
export const WASM_RUNTIME_OVERRIDES = process.env.WASM_RUNTIME_OVERRIDES || "";

// Weight per step mapping
export const WEIGHT_PER_STEP = 1_000_000_000_000n / 40_000_000n;
