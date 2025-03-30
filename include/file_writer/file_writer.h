#ifndef FILE_WRITER_H
#define FILE_WRITER_H

#include <stddef.h> // for size_t
#include <stdint.h> // for uint8_t

#ifdef __cplusplus
extern "C" {
#endif

typedef enum FileWriterError {
    Success = 0,
    FileOpenError = 1,
    FileWriteError = 2,
    FileCloseError = 3,
    InvalidHandle = 4,
    InvalidPath = 5,
    InvalidData = 6,
    IoError = 7,
} FileWriterError;

typedef enum FileWriterMode {
    Append = 0,
    Write = 1,
} FileWriterMode;


// --- Opaque Handle ---

// Forward declare the opaque struct. C/C++ code only deals with pointers to it.
typedef struct FileWriterHandle FileWriterHandle;


// --- Functions ---

/**
 * @brief Creates a new file writer instance.
 *
 * Opens the specified file path in the given mode. On success, allocates a new
 * FileWriterHandle and stores its pointer in the location pointed to by `handle`.
 *
 * @param path Null-terminated UTF-8 encoded file path.
 * @param handle Output parameter: Pointer to a FileWriterHandle pointer. Will be set to the new handle on success.
 * @param mode File opening mode (Append or Write/Overwrite).
 * @return FileWriterError::Success on success, or an error code otherwise.
 *         On error, *handle will be set to NULL.
 */
FileWriterError file_writer_new(const char* path, FileWriterHandle** handle, FileWriterMode mode);

/**
 * @brief Sets the internal buffer capacity of the writer.
 *
 * Recreates the internal buffer with the specified size. Existing data in the
 * buffer will be flushed to the file before resizing.
 * Performance Note: Frequent resizing can be inefficient. Set a reasonable size initially if possible.
 *
 * @param handle A valid handle obtained from file_writer_new.
 * @param size The desired buffer capacity in bytes (must be > 0).
 * @return FileWriterError::Success on success, or an error code otherwise.
 */
FileWriterError file_writer_set_buffer_size(FileWriterHandle* handle, size_t size);

/**
 * @brief Writes raw byte data to the buffered file writer.
 *
 * Data may not be written to the underlying file immediately; call file_writer_flush
 * or file_writer_close to ensure data is persisted.
 *
 * @param handle A valid handle obtained from file_writer_new.
 * @param data Pointer to the byte buffer to write.
 * @param size Number of bytes to write from the buffer.
 * @return FileWriterError::Success on success, or an error code otherwise.
 */
FileWriterError file_writer_write_raw(FileWriterHandle* handle, const uint8_t* data, size_t size);

/**
 * @brief Writes a null-terminated string to the buffered file writer.
 *
 * The null terminator itself is NOT written to the file.
 * Data may not be written to the underlying file immediately; call file_writer_flush
 * or file_writer_close to ensure data is persisted.
 *
 * @param handle A valid handle obtained from file_writer_new.
 * @param str Pointer to the null-terminated string to write.
 * @return FileWriterError::Success on success, or an error code otherwise.
 */
FileWriterError file_writer_write_string(FileWriterHandle* handle, const char* str);

/**
 * @brief Flushes the internal buffer to the underlying file.
 *
 * Ensures that all data previously written using write functions is attempted
 * to be written to the OS/disk.
 *
 * @param handle A valid handle obtained from file_writer_new.
 * @return FileWriterError::Success on success, or an error code otherwise (e.g., disk full).
 */
FileWriterError file_writer_flush(FileWriterHandle* handle);

/**
 * @brief Flushes the buffer, closes the file, and releases all resources associated with the handle.
 *
 * After calling this function, the handle is invalid and must not be used again.
 * It is crucial to call this function to ensure data is fully written and resources are freed.
 *
 * @param handle A valid handle obtained from file_writer_new.
 * @return FileWriterError::Success on success, or FileWriterError::FileCloseError if flushing failed.
 *         The handle is invalidated even if an error occurs.
 */
FileWriterError file_writer_close(FileWriterHandle* handle);


#ifdef __cplusplus
} // extern "C"
#endif

#endif // FILE_WRITER_H