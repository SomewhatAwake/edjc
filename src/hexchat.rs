/*!
HexChat plugin bindings and utilities.

This module provides the necessary FFI bindings and helper functions
to interface with the HexChat plugin API.
*/

use libc::{c_char, c_int, c_void};
use log::info;

/// HexChat plugin handle (opaque pointer)
pub type HexChatPlugin = c_void;

/// HexChat hook return values
pub const HEXCHAT_EAT_NONE: c_int = 0; // Don't eat this event, pass it on
#[allow(dead_code)]
pub const HEXCHAT_EAT_HEXCHAT: c_int = 1; // Don't let HexChat see this event
#[allow(dead_code)]
pub const HEXCHAT_EAT_PLUGIN: c_int = 2; // Don't let other plugins see this event
#[allow(dead_code)]
pub const HEXCHAT_EAT_ALL: c_int = 3; // Don't let anything see this event

/// HexChat context handle
#[allow(dead_code)]
pub type HexChatContext = c_void;

/// HexChat hook handle  
#[allow(dead_code)]
pub type HexChatHook = c_void;

/// Function pointer type for HexChat callbacks
#[allow(dead_code)]
pub type HexChatCallback = extern "C" fn(
    word: *const *const c_char,
    word_eol: *const *const c_char,
    user_data: *mut c_void,
) -> c_int;

// HexChat API function pointers - these will be provided by HexChat at runtime
#[allow(dead_code)]
static mut HEXCHAT_PRINT: Option<unsafe extern "C" fn(*mut HexChatPlugin, *const c_char)> = None;
#[allow(dead_code)]
static mut HEXCHAT_HOOK_PRINT: Option<
    unsafe extern "C" fn(
        *mut HexChatPlugin,
        *const c_char,
        Option<HexChatCallback>,
        *mut c_void,
    ) -> *mut HexChatHook,
> = None;
#[allow(dead_code)]
static mut HEXCHAT_HOOK_COMMAND: Option<
    unsafe extern "C" fn(
        *mut HexChatPlugin,
        *const c_char,
        Option<HexChatCallback>,
        *mut c_void,
    ) -> *mut HexChatHook,
> = None;
#[allow(dead_code)]
static mut HEXCHAT_COMMAND: Option<unsafe extern "C" fn(*mut HexChatPlugin, *const c_char)> = None;
#[allow(dead_code)]
static mut HEXCHAT_GET_INFO: Option<
    unsafe extern "C" fn(*mut HexChatPlugin, *const c_char) -> *const c_char,
> = None;
#[allow(dead_code)]
static mut HEXCHAT_UNHOOK: Option<
    unsafe extern "C" fn(*mut HexChatPlugin, *mut HexChatHook) -> *mut c_void,
> = None;

// Global plugin handle storage
static mut PLUGIN_HANDLE: *mut HexChatPlugin = std::ptr::null_mut();

/// Store the plugin handle for later use
pub fn store_plugin_handle(handle: *mut HexChatPlugin) {
    unsafe {
        PLUGIN_HANDLE = handle;
    }
}

/// Initialize HexChat API function pointers
/// This should be called from hexchat_plugin_init with the provided function pointers
#[allow(dead_code)]
pub unsafe fn init_hexchat_api(
    plugin_handle: *mut HexChatPlugin,
    print_fn: unsafe extern "C" fn(*mut HexChatPlugin, *const c_char),
    hook_print_fn: unsafe extern "C" fn(
        *mut HexChatPlugin,
        *const c_char,
        Option<HexChatCallback>,
        *mut c_void,
    ) -> *mut HexChatHook,
    command_fn: unsafe extern "C" fn(*mut HexChatPlugin, *const c_char),
    get_info_fn: unsafe extern "C" fn(*mut HexChatPlugin, *const c_char) -> *const c_char,
    unhook_fn: unsafe extern "C" fn(*mut HexChatPlugin, *mut HexChatHook) -> *mut c_void,
) {
    PLUGIN_HANDLE = plugin_handle;
    HEXCHAT_PRINT = Some(print_fn);
    HEXCHAT_HOOK_PRINT = Some(hook_print_fn);
    HEXCHAT_COMMAND = Some(command_fn);
    HEXCHAT_GET_INFO = Some(get_info_fn);
    HEXCHAT_UNHOOK = Some(unhook_fn);
}

/// Print text to HexChat
pub fn hexchat_print(text: *const c_char) {
    unsafe {
        if let Some(print_fn) = HEXCHAT_PRINT {
            if !PLUGIN_HANDLE.is_null() {
                print_fn(PLUGIN_HANDLE, text);
                return;
            }
        }

        // Fallback for testing/debugging - print to stderr so it's visible
        eprintln!("HEXCHAT: {}", c_str_to_string(text));
    }
}

/// Hook into a HexChat print event
#[allow(dead_code)]
pub fn hexchat_hook_print(
    name: *const c_char,
    _callback: Option<HexChatCallback>,
    _user_data: *mut c_void,
) -> *mut HexChatHook {
    // For now, return a dummy hook - in a real implementation,
    // this would use the proper HexChat API
    info!("Hook registered for: {}", c_str_to_string(name));
    std::ptr::null_mut()
}

/// Hook into a HexChat command
#[allow(dead_code)]
pub fn hexchat_hook_command(
    name: *const c_char,
    callback: Option<HexChatCallback>,
    user_data: *mut c_void,
) -> *mut HexChatHook {
    unsafe {
        if let Some(hook_command_fn) = HEXCHAT_HOOK_COMMAND {
            if !PLUGIN_HANDLE.is_null() {
                return hook_command_fn(PLUGIN_HANDLE, name, callback, user_data);
            }
        }

        // Fallback for testing/debugging
        info!("Command hook registered for: {}", c_str_to_string(name));
        std::ptr::null_mut()
    }
}

/// Send a command to HexChat
#[allow(dead_code)]
pub fn hexchat_command(command: *const c_char) {
    info!("Command: {}", c_str_to_string(command));
    // Simplified implementation
}

/// Get HexChat information
#[allow(dead_code)]
pub fn hexchat_get_info(id: *const c_char) -> *const c_char {
    info!("Info requested: {}", c_str_to_string(id));
    std::ptr::null()
}

/// Unhook a previously hooked event
#[allow(dead_code)]
pub fn hexchat_unhook(hook: *mut HexChatHook) -> *mut c_void {
    info!("Unhooking: {hook:?}");
    std::ptr::null_mut()
}

/// Utility function to safely convert C strings
pub fn c_str_to_string(c_str: *const c_char) -> String {
    if c_str.is_null() {
        return String::new();
    }

    unsafe {
        std::ffi::CStr::from_ptr(c_str)
            .to_string_lossy()
            .into_owned()
    }
}

/// Utility function to create C strings safely
#[allow(dead_code)]
pub fn string_to_c_str(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap_or_else(|_| std::ffi::CString::new("").unwrap())
}
