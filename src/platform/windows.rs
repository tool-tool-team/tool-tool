use crate::config::ToolConfiguration;
use crate::platform::Platform;

#[cfg(target_os = "windows")]
use crate::util::retry;
#[cfg(target_os = "windows")]
use winapi::um::consoleapi::SetConsoleCtrlHandler;
#[cfg(target_os = "windows")]
use winapi::um::wincon::{CTRL_C_EVENT};
#[cfg(target_os = "windows")]
use winapi::shared::minwindef::{TRUE, DWORD, BOOL, FALSE};

pub struct Windows;

#[cfg(target_os = "windows")]
impl crate::platform::PlatformFunctions for Windows {
    fn rename_atomically(src: &std::path::Path, dst: &std::path::Path) -> crate::Result<()> {
        retry(|| atomicwrites::move_atomic(src, dst))
    }
}

impl Platform for Windows {
    fn get_download_url<'a>(&self, tool_configuration: &'a ToolConfiguration) -> Option<&'a str> {
        tool_configuration
            .download
            .windows
            .as_deref()
            .or_else(|| tool_configuration.download.default.as_deref())
    }

    fn get_application_extensions(&self) -> &'static [&'static str] {
        &[".exe", ".cmd", ".bat", ""]
    }

    fn get_name(&self) -> &'static str {
        "windows"
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn install_control_handler() {
    if 0 != unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) } {
        // Control handler installed
    } else {
        println!("WARNING: Could not set control handler.");
    }
}

#[cfg(target_os = "windows")]
extern "system" fn ctrl_handler(ctrl_type: DWORD) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT => {
            // Do not exit automatically, let event propagate to child process
            TRUE
        }
        _ => FALSE,
    }
}