/*!
Build script for EDJC HexChat plugin.

This script handles any build-time configuration and ensures
the plugin is built correctly for the target platform.
*/

use std::env;

fn main() {
    // Tell cargo to rerun this script if any of these files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Set up platform-specific configuration
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    match target_os.as_str() {
        "windows" => {
            // On Windows, HexChat plugins are typically .dll files
            // Export symbols are handled by #[no_mangle] attributes
            println!("cargo:rustc-link-arg=/EXPORT:hexchat_plugin_init");
            println!("cargo:rustc-link-arg=/EXPORT:hexchat_plugin_deinit");
        }
        "linux" => {
            // On Linux, HexChat plugins are .so files
            // Ensure we export the required symbols
            println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
        }
        "macos" => {
            // On macOS, HexChat plugins are .so files (not .dylib)
            println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
        }
        _ => {
            println!("cargo:warning=Unknown target OS: {target_os}");
        }
    }

    // Add version information
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    println!("cargo:rustc-env=PLUGIN_VERSION={version}");

    let name = env::var("CARGO_PKG_NAME").unwrap();
    println!("cargo:rustc-env=PLUGIN_NAME={name}");
}
