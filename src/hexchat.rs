use libc::{c_char, c_int, c_void};
use std::ffi::CStr;

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

// Global plugin handle storage
static mut PLUGIN_HANDLE: *mut HexChatPlugin = std::ptr::null_mut();

/// Store the plugin handle for later use
pub fn store_plugin_handle(handle: *mut HexChatPlugin) {
    unsafe {
        PLUGIN_HANDLE = handle;
    }
}

/// Simple API initialization that just stores the handle
pub unsafe fn init_hexchat_api_from_arg(
    plugin_handle: *mut HexChatPlugin,
    _arg: *const c_char,
) -> bool {
    PLUGIN_HANDLE = plugin_handle;
    true
}

/// Print text to HexChat - for now just use stderr which shows in HexChat
pub fn hexchat_print(text: *const c_char) {
    unsafe {
        if !text.is_null() {
            if let Ok(text_str) = CStr::from_ptr(text).to_str() {
                // Use eprintln! which will appear in HexChat's console
                eprintln!("[EDJC] {}", text_str);
            }
        }
    }
}

/// Register a command hook - disabled for now to prevent crashes
pub fn hexchat_hook_command(
    name: *const c_char,
    _callback: Option<HexChatCallback>,
    _user_data: *mut c_void,
) -> *mut HexChatHook {
    unsafe {
        let cmd_name = if !name.is_null() {
            CStr::from_ptr(name).to_string_lossy().into_owned()
        } else {
            "unknown".to_string()
        };
        
        // For now, just log that we would register the command
        eprintln!("[EDJC] Would register command hook for: {}", cmd_name);
        eprintln!("[EDJC] Command hooks temporarily disabled for stability");
        
        // Return a dummy hook pointer
        1 as *mut HexChatHook
    }
}

/// Utility function to safely convert C strings
pub fn c_str_to_string(c_str: *const c_char) -> String {
    if c_str.is_null() {
        return String::new();
    }

    unsafe {
        CStr::from_ptr(c_str)
            .to_string_lossy()
            .into_owned()
    }
}
