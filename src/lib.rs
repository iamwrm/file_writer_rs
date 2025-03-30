use std::ffi::{c_char, CStr};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, ErrorKind, Write};
use std::path::Path;
use std::ptr::null_mut;
use std::slice;

// Define C-compatible error enum. Using repr(C) ensures predictable layout.
// We use isize as the underlying type for better C compatibility than Rust's default.
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum FileWriterError {
    Success = 0,
    FileOpenError = 1,
    FileWriteError = 2,
    FileCloseError = 3, // Error during explicit flush/close
    InvalidHandle = 4,  // Null handle passed
    InvalidPath = 5,    // Null or invalid UTF-8 path
    InvalidData = 6,    // Null data pointer passed
    IoError = 7,        // Generic I/O error not covered above
}

impl From<std::io::Error> for FileWriterError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            // Add more specific mappings if needed
            ErrorKind::PermissionDenied => FileWriterError::FileOpenError,
            ErrorKind::NotFound => FileWriterError::FileOpenError,
            _ => FileWriterError::IoError, // Catch-all for other IO errors
        }
    }
}

// Define C-compatible mode enum
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum FileWriterMode {
    Append = 0,
    Write = 1, // Overwrite/Create
}

// Opaque struct in Rust holding the actual writer
pub struct FileWriter {
    // Using Option allows us to potentially take ownership during close/resize
    // for better error handling, though Box<BufWriter> is simpler if resizing isn't needed.
    // Let's stick with Box<BufWriter> for simplicity first.
    writer: Option<BufWriter<File>>,
}

// The C FFI Handle is just a raw pointer to our Rust struct
pub type FileWriterHandle = FileWriter;

// Helper function to safely get a mutable reference to the writer
// Returns InvalidHandle error if the pointer is null or writer is None.
#[inline]
fn get_writer_mut(
    handle: *mut FileWriterHandle,
) -> Result<&'static mut BufWriter<File>, FileWriterError> {
    // Cast to *mut FileWriter and check null.
    // This is 'static because the borrow lasts only for the function call,
    // and we are careful about aliasing rules via the FFI boundary.
    // This is safe because C guarantees sequential calls for a given handle.
    unsafe {
        handle
            .as_mut()
            .and_then(|fw| fw.writer.as_mut())
            .ok_or(FileWriterError::InvalidHandle)
    }
}

/// Creates a new file writer.
///
/// # Safety
/// - `path` must be a valid, null-terminated C string representing a valid path.
/// - `handle` must be a valid pointer to a `FileWriterHandle*`.
/// - The memory pointed to by `path` must be valid for the duration of the call.
/// - The memory pointed to by `handle` must be valid for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_writer_new(
    path: *const c_char,
    handle: *mut *mut FileWriterHandle,
    mode: FileWriterMode,
) -> FileWriterError {
    if path.is_null() {
        return FileWriterError::InvalidPath;
    }
    if handle.is_null() {
        // Cannot return the handle if the handle pointer itself is null
        return FileWriterError::InvalidHandle;
    }
    // Initialize the output handle to null in case of errors
    unsafe { *handle = null_mut() };

    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return FileWriterError::InvalidPath, // UTF-8 conversion error
    };

    let path_obj = Path::new(path_str);

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

    // Default buffer size is typically 8KB, which is usually good.
    // We will add set_buffer_size later.
    let writer = BufWriter::new(file);

    let file_writer = FileWriter {
        writer: Some(writer),
    };

    // Allocate memory on the heap for the FileWriter struct and return a raw pointer
    let boxed_writer = Box::new(file_writer);
    unsafe {
        *handle = Box::into_raw(boxed_writer);
    }

    FileWriterError::Success
}

/// Sets the buffer capacity. This recreates the underlying BufWriter.
/// Any data currently in the buffer will be flushed before resizing.
///
/// # Safety
/// - `handle` must be a valid pointer obtained from `file_writer_new` and not yet closed.
/// - `size` should be greater than 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_writer_set_buffer_size(
    handle: *mut FileWriterHandle,
    size: usize,
) -> FileWriterError {
    if size == 0 {
        // BufWriter panics if capacity is 0
        return FileWriterError::InvalidData;
    }

    let file_writer = unsafe {
        match handle.as_mut() {
            Some(fw) => fw,
            None => return FileWriterError::InvalidHandle,
        }
    };

    // Take the old writer out of the Option
    let old_writer = match file_writer.writer.take() {
        Some(w) => w,
        None => return FileWriterError::InvalidHandle, // Should not happen if handle is valid
    };

    // into_inner() flushes the buffer and returns the inner File
    match old_writer.into_inner() {
        Ok(file) => {
            // Create a new BufWriter with the desired capacity
            let new_writer = BufWriter::with_capacity(size, file);
            // Put the new writer back into the FileWriter struct
            file_writer.writer = Some(new_writer);
            FileWriterError::Success
        }
        Err(_e) => {
            // If into_inner fails, the file handle is lost in the error.
            // We cannot recover the original writer state easily. Mark handle as unusable.
            // The original file writer remains None.
            FileWriterError::FileCloseError // Error occurred during flush (part of into_inner)
        }
    }
}

/// Writes raw bytes to the buffered writer.
///
/// # Safety
/// - `handle` must be a valid pointer obtained from `file_writer_new` and not yet closed.
/// - `data` must be a valid pointer to a byte buffer of at least `size` bytes.
/// - The memory pointed to by `data` must be valid for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_writer_write_raw(
    handle: *mut FileWriterHandle,
    data: *const u8,
    size: usize,
) -> FileWriterError {
    if size > 0 && data.is_null() {
        return FileWriterError::InvalidData;
    }

    let writer = match get_writer_mut(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };

    // Create a slice from the raw parts. This is unsafe because we trust
    // the C caller provided valid data pointer and size.
    let data_slice = unsafe { slice::from_raw_parts(data, size) };

    match writer.write_all(data_slice) {
        Ok(_) => FileWriterError::Success,
        Err(_) => FileWriterError::FileWriteError,
    }
}

/// Writes a null-terminated C string to the buffered writer.
///
/// # Safety
/// - `handle` must be a valid pointer obtained from `file_writer_new` and not yet closed.
/// - `str` must be a valid, null-terminated C string.
/// - The memory pointed to by `str` must be valid for the duration of the call.
#[unsafe(no_mangle)]
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
    let bytes = c_str.to_bytes(); // No null terminator included

    match writer.write_all(bytes) {
        Ok(_) => FileWriterError::Success,
        Err(_) => FileWriterError::FileWriteError,
    }
}

/// Flushes the buffer, ensuring all data written so far is passed to the OS.
///
/// # Safety
/// - `handle` must be a valid pointer obtained from `file_writer_new` and not yet closed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_writer_flush(handle: *mut FileWriterHandle) -> FileWriterError {
    let writer = match get_writer_mut(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };

    match writer.flush() {
        Ok(_) => FileWriterError::Success,
        // Flushing is a form of writing/IO
        Err(_) => FileWriterError::FileWriteError,
    }
}

/// Flushes the buffer, closes the file, and frees the writer's resources.
/// The handle becomes invalid after this call.
///
/// # Safety
/// - `handle` must be a valid pointer obtained from `file_writer_new` and not yet closed.
/// - `handle` must not be used after this function is called.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_writer_close(handle: *mut FileWriterHandle) -> FileWriterError {
    if handle.is_null() {
        // Attempting to close a null handle could be considered success (noop) or error.
        // Let's treat it as an error to catch potential bugs in C code.
        return FileWriterError::InvalidHandle;
    }

    // Convert the raw pointer back into a Box. This transfers ownership back to Rust.
    // The Box will be dropped at the end of this scope.
    let boxed_writer = unsafe { Box::from_raw(handle) };

    // Explicitly take the writer and drop it (which flushes and closes).
    // This allows catching errors specifically from the final flush/close.
    // Dropping the Box<FileWriter> would do this implicitly, but explicit drop
    // of the BufWriter gives us a chance to check the result.
    if let Some(writer) = boxed_writer.writer {
        // into_inner flushes before returning the File. The File is then dropped.
        match writer.into_inner() {
            Ok(_file) => {
                // File is dropped here, closing it.
                FileWriterError::Success
            }
            Err(_) => {
                // Error during the final flush. The handle is still consumed.
                FileWriterError::FileCloseError
            }
        }
        // Note: Even on error, the Box is dropped, freeing the memory of FileWriter.
    } else {
        // This case might happen if set_buffer_size failed and left writer as None
        FileWriterError::InvalidHandle // Or maybe Success? Let's indicate something went wrong before.
    }
}
