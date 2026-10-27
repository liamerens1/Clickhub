use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_BINARY, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW,
    RegSetKeyValueW,
};
use windows::core::{HSTRING, PCWSTR, w};

/// Per-user programs started at sign-in. Only the current user's key is ever touched: ClickHub
/// never writes machine-wide settings.
const RUN_KEY: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
/// Where Task Manager's Startup tab records that the user switched an entry off.
const APPROVED_KEY: PCWSTR =
    w!(r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run");

/// Whether `name` starts at sign-in: it has a value in the Run key and Task Manager has not
/// switched it off.
///
/// # Errors
///
/// A message with Windows' error code when the registry cannot be read.
pub fn run_entry_active(name: &str) -> Result<bool, String> {
    let value = HSTRING::from(name);
    let mut size = 0_u32;
    // SAFETY: valid key path and value name; only the data size is requested.
    let found = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            RUN_KEY,
            &value,
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&raw mut size),
        )
    };
    if found == ERROR_FILE_NOT_FOUND {
        return Ok(false);
    }
    check(found, "read the startup entry")?;
    Ok(!switched_off_in_task_manager(&value)?)
}

/// Starts `command` (a quoted exe path plus arguments) at sign-in under `name`, and clears any
/// "disabled" mark Task Manager left for it, so switching it on in ClickHub really switches it
/// on.
///
/// # Errors
///
/// A message with Windows' error code when the registry cannot be written.
pub fn set_run_entry(name: &str, command: &str) -> Result<(), String> {
    let value = HSTRING::from(name);
    let data: Vec<u16> = command.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = u32::try_from(data.len() * 2).map_err(|_| "startup command is too long")?;
    // SAFETY: `data` is a null-terminated UTF-16 string of `bytes` bytes that outlives the call.
    let written = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            RUN_KEY,
            &value,
            REG_SZ.0,
            Some(data.as_ptr().cast()),
            bytes,
        )
    };
    check(written, "write the startup entry")?;
    delete_value(
        APPROVED_KEY,
        &value,
        "clear the Task Manager startup setting",
    )
}

/// Stops `name` starting at sign-in. Removing an entry that does not exist is not an error.
///
/// # Errors
///
/// A message with Windows' error code when the registry cannot be written.
pub fn remove_run_entry(name: &str) -> Result<(), String> {
    let value = HSTRING::from(name);
    delete_value(RUN_KEY, &value, "remove the startup entry")?;
    delete_value(
        APPROVED_KEY,
        &value,
        "clear the Task Manager startup setting",
    )
}

/// Task Manager stores 12 bytes per entry; a disabled entry carries the time it was disabled in
/// the last 8 (all zero when enabled).
fn switched_off_in_task_manager(value: &HSTRING) -> Result<bool, String> {
    let mut data = [0_u8; 32];
    let mut size = 32_u32;
    // SAFETY: `data` has room for `size` bytes and outlives the call.
    let read = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            APPROVED_KEY,
            value,
            RRF_RT_REG_BINARY,
            None,
            Some(data.as_mut_ptr().cast()),
            Some(&raw mut size),
        )
    };
    if read == ERROR_FILE_NOT_FOUND {
        return Ok(false);
    }
    check(read, "read the Task Manager startup setting")?;
    let stored = data.get(..size as usize).unwrap_or(&data);
    Ok(stored.len() >= 8 && stored.iter().rev().take(8).any(|byte| *byte != 0))
}

fn delete_value(key: PCWSTR, value: &HSTRING, what: &str) -> Result<(), String> {
    // SAFETY: valid key path and value name.
    let deleted = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key, value) };
    if deleted == ERROR_FILE_NOT_FOUND {
        return Ok(());
    }
    check(deleted, what)
}

fn check(result: WIN32_ERROR, what: &str) -> Result<(), String> {
    if result == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("could not {what} (Windows error {})", result.0))
    }
}
