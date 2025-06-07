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


typedef struct FileWriterHandle FileWriterHandle;

FileWriterError file_writer_new(const char* path, FileWriterHandle** handle, FileWriterMode mode);

FileWriterError file_writer_set_buffer_size(FileWriterHandle* handle, size_t size);

FileWriterError file_writer_write_raw(FileWriterHandle* handle, const uint8_t* data, size_t size);

FileWriterError file_writer_write_string(FileWriterHandle* handle, const char* str);

FileWriterError file_writer_flush(FileWriterHandle* handle);

typedef struct BufferDescriptor {
    const uint8_t* data;
    size_t size;
} BufferDescriptor;

FileWriterError file_writer_write_batch(FileWriterHandle* handle, const BufferDescriptor* buffers, size_t count);

FileWriterError file_writer_write_large(FileWriterHandle* handle, const uint8_t* data, size_t size);

FileWriterError file_writer_close(FileWriterHandle* handle);


#ifdef __cplusplus
} // extern "C"
#endif

#endif // FILE_WRITER_H