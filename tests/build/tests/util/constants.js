"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.WEIGHT_PER_STEP = exports.WASM_RUNTIME_OVERRIDES = exports.SPAWNING_TIME = exports.OVERRIDE_RUNTIME_PATH = exports.BINARY_PATH = exports.DEBUG_MODE = exports.MADARA_LOG = exports.DISPLAY_LOG = exports.CUSTOM_SPEC_PATH = exports.BASE_PATH = void 0;
exports.BASE_PATH = process.env.BASE_PATH;
exports.CUSTOM_SPEC_PATH = process.env.CUSTOM_SPEC_PATH;
exports.DISPLAY_LOG = process.env.DISPLAY_LOG || false;
exports.MADARA_LOG = process.env.MADARA_LOG || "info";
exports.DEBUG_MODE = process.env.DEBUG_MODE || false;
exports.BINARY_PATH = process.env.BINARY_PATH || "../target/release/madara";
exports.OVERRIDE_RUNTIME_PATH = process.env.OVERRIDE_RUNTIME_PATH || undefined;
exports.SPAWNING_TIME = 500000;
exports.WASM_RUNTIME_OVERRIDES = process.env.WASM_RUNTIME_OVERRIDES || "";
exports.WEIGHT_PER_STEP = 1000000000000n / 40000000n;
//# sourceMappingURL=constants.js.map