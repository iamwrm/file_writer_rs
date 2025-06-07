use std::ffi::{c_char, CStr};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, ErrorKind, Write};
use std::path::Path;
use std::ptr::null_mut;
use std::slice;

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum FileWriterError {
    Success = 0,
    FileOpenError = 1,
    FileWriteError = 2,
    FileCloseError = 3, // Error during explicit flush/close
    InvalidHandle = 4,
    InvalidPath = 5,
    InvalidData = 6,
    IoError = 7,
}

impl From<std::io::Error> for FileWriterError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            ErrorKind::PermissionDenied => FileWriterError::FileOpenError,
            ErrorKind::NotFound => FileWriterError::FileOpenError,
            _ => FileWriterError::IoError,
        }
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum FileWriterMode {
    Append = 0,
    Write = 1,
}

pub struct FileWriter {
    writer: Option<BufWriter<File>>,
    is_valid: bool,
}

pub type FileWriterHandle = FileWriter;

#[inline(always)]
fn get_writer_mut(
    handle: *mut FileWriterHandle,
) -> Result<&'static mut BufWriter<File>, FileWriterError> {
    unsafe {
        if !handle.is_null() {
            let fw = &mut *handle;
            if fw.is_valid {
                if let Some(ref mut writer) = fw.writer {
                    return Ok(writer);
                }
            }
        }
        Err(FileWriterError::InvalidHandle)
    }
}

/// # Safety
/// - `path` must be a valid null-terminated C string
/// - `handle` must be a valid pointer to store the result
#[no_mangle]
pub unsafe extern "C" fn file_writer_new(
    path: *const c_char,
    handle: *mut *mut FileWriterHandle,
    mode: FileWriterMode,
) -> FileWriterError {
    if path.is_null() {
        return FileWriterError::InvalidPath;
    }
    if handle.is_null() {
        return FileWriterError::InvalidHandle;
    }
    unsafe { *handle = null_mut() };

    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return FileWriterError::InvalidPath,
    };

    let path_obj = Path::new(path_str);

    if let Some(parent) = path_obj.parent() {
        if !parent.as_os_str().is_empty() && std::fs::create_dir_all(parent).is_err() {
            return FileWriterError::FileOpenError;
        }
    }

    let file_result = match mode {
        FileWriterMode::Append => OpenOptions::new().create(true).append(true).open(path_obj),
        FileWriterMode::Write => OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path_obj),
    };

    let file = match file_result {
        Ok(f) => f,
        Err(_) => return FileWriterError::FileOpenError,
    };

    let writer = BufWriter::with_capacity(64 * 1024, file);

    let file_writer = FileWriter {
        writer: Some(writer),
        is_valid: true,
    };

    let boxed_writer = Box::new(file_writer);
    unsafe {
        *handle = Box::into_raw(boxed_writer);
    }

    FileWriterError::Success
}

/// # Safety
/// - `handle` must be a valid FileWriterHandle pointer
#[no_mangle]
pub unsafe extern "C" fn file_writer_set_buffer_size(
    handle: *mut FileWriterHandle,
    size: usize,
) -> FileWriterError {
    if size == 0 {
        return FileWriterError::InvalidData;
    }

    let file_writer = unsafe {
        match handle.as_mut() {
            Some(fw) => fw,
            None => return FileWriterError::InvalidHandle,
        }
    };

    let old_writer = match file_writer.writer.take() {
        Some(w) => w,
        None => return FileWriterError::InvalidHandle,
    };

    match old_writer.into_inner() {
        Ok(file) => {
            let new_writer = BufWriter::with_capacity(size, file);
            file_writer.writer = Some(new_writer);
            file_writer.is_valid = true;
            FileWriterError::Success
        }
        Err(_e) => {
            file_writer.is_valid = false;
            FileWriterError::FileCloseError
        }
    }
}

/// # Safety
/// - `handle` must be a valid FileWriterHandle pointer
/// - `data` must point to valid memory of at least `size` bytes
#[no_mangle]
pub unsafe extern "C" fn file_writer_write_raw(
    handle: *mut FileWriterHandle,
    data: *const u8,
    size: usize,
) -> FileWriterError {
    if size == 0 {
        return FileWriterError::Success;
    }

    if data.is_null() {
        return FileWriterError::InvalidData;
    }

    let writer = match get_writer_mut(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };

    let data_slice = unsafe { slice::from_raw_parts(data, size) };

    match writer.write_all(data_slice) {
        Ok(_) => FileWriterError::Success,
        Err(_) => FileWriterError::FileWriteError,
    }
}

/// # Safety
/// - `handle` must be a valid FileWriterHandle pointer
/// - `str_ptr` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn file_writer_write_string(
    handle: *mut FileWriterHandle,
    str_ptr: *const c_char,
) -> FileWriterError {
    if str_ptr.is_null() {
        return FileWriterError::InvalidData;
    }

    let writer = match get_writer_mut(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };

    let c_str = unsafe { CStr::from_ptr(str_ptr) };
    let bytes = c_str.to_bytes();

    match writer.write_all(bytes) {
        Ok(_) => FileWriterError::Success,
        Err(_) => FileWriterError::FileWriteError,
    }
}

/// # Safety
/// - `handle` must be a valid FileWriterHandle pointer
#[no_mangle]
pub unsafe extern "C" fn file_writer_flush(handle: *mut FileWriterHandle) -> FileWriterError {
    let writer = match get_writer_mut(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };

    match writer.flush() {
        Ok(_) => FileWriterError::Success,
        Err(_) => FileWriterError::FileWriteError,
    }
}

#[repr(C)]
pub struct BufferDescriptor {
    pub data: *const u8,
    pub size: usize,
}

/// # Safety
/// - `handle` must be a valid FileWriterHandle pointer
/// - `buffers` must point to valid BufferDescriptor array of `count` elements
#[no_mangle]
pub unsafe extern "C" fn file_writer_write_batch(
    handle: *mut FileWriterHandle,
    buffers: *const BufferDescriptor,
    count: usize,
) -> FileWriterError {
    if buffers.is_null() {
        return FileWriterError::InvalidData;
    }

    if count == 0 {
        return FileWriterError::Success;
    }

    let writer = match get_writer_mut(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };

    let buffer_slice = unsafe { slice::from_raw_parts(buffers, count) };

    for buffer in buffer_slice {
        if buffer.size > 0 {
            if buffer.data.is_null() {
                return FileWriterError::InvalidData;
            }
            let data_slice = unsafe { slice::from_raw_parts(buffer.data, buffer.size) };
            if writer.write_all(data_slice).is_err() {
                return FileWriterError::FileWriteError;
            }
        }
    }

    FileWriterError::Success
}

/// # Safety
/// - `handle` must be a valid FileWriterHandle pointer
/// - `data` must point to valid memory of at least `size` bytes
#[no_mangle]
pub unsafe extern "C" fn file_writer_write_large(
    handle: *mut FileWriterHandle,
    data: *const u8,
    size: usize,
) -> FileWriterError {
    // Fast path for empty writes
    if size == 0 {
        return FileWriterError::Success;
    }

    if data.is_null() {
        return FileWriterError::InvalidData;
    }

    let writer = match get_writer_mut(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };

    let data_slice = unsafe { slice::from_raw_parts(data, size) };

    if size > 1024 * 1024 {
        if writer.flush().is_err() {
            return FileWriterError::FileWriteError;
        }

        if writer.get_mut().write_all(data_slice).is_err() {
            return FileWriterError::FileWriteError;
        }
    } else if writer.write_all(data_slice).is_err() {
        return FileWriterError::FileWriteError;
    }

    FileWriterError::Success
}

/// # Safety
/// - `handle` must be a valid FileWriterHandle pointer
#[no_mangle]
pub unsafe extern "C" fn file_writer_close(handle: *mut FileWriterHandle) -> FileWriterError {
    if handle.is_null() {
        return FileWriterError::InvalidHandle;
    }

    let boxed_writer = unsafe { Box::from_raw(handle) };

    if let Some(writer) = boxed_writer.writer {
        match writer.into_inner() {
            Ok(_file) => FileWriterError::Success,
            Err(_) => FileWriterError::FileCloseError,
        }
    } else {
        FileWriterError::InvalidHandle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use tempfile::TempDir;

    #[test]
    fn test_create_directories() {
        // Create a temporary directory that will be automatically cleaned up
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Create a path with non-existent subdirectories
        let subdir_path = base_path.join("a/b/c");
        let file_path = subdir_path.join("test.txt");

        // Convert path to C string
        let c_path =
            CString::new(file_path.to_string_lossy().as_bytes()).expect("Failed to create CString");

        let mut handle: *mut FileWriterHandle = std::ptr::null_mut();
        let handle_ptr: *mut *mut FileWriterHandle = &mut handle;

        // Call the function being tested
        unsafe {
            let result = file_writer_new(c_path.as_ptr(), handle_ptr, FileWriterMode::Write);

            // Verify success
            assert_eq!(result, FileWriterError::Success);
            assert!(!handle.is_null());

            // Verify directories and file were created
            assert!(subdir_path.exists());
            assert!(file_path.exists());

            // Clean up
            file_writer_close(handle);
        }

        // Directory should still exist after closing the file
        assert!(subdir_path.exists());
    }
}
