use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use file_writer::{
    file_writer_close, file_writer_new, file_writer_write_raw, file_writer_write_string,
    FileWriterError, FileWriterHandle, FileWriterMode,
};
use std::{ffi::CString, fs, io::Write, ptr::null_mut};
use tempfile::NamedTempFile;

const KIB: usize = 1024;
const MIB: usize = 1024 * KIB;

// Helper to create a handle for benchmarks
fn setup_writer(path: &str, mode: FileWriterMode) -> (*mut FileWriterHandle, CString) {
    let c_path = CString::new(path).expect("CString::new failed");
    let mut handle: *mut FileWriterHandle = null_mut();
    let result = unsafe { file_writer_new(c_path.as_ptr(), &mut handle, mode) };
    assert_eq!(result, FileWriterError::Success, "Failed to create writer");
    assert!(!handle.is_null(), "Handle is null after creation");
    (handle, c_path) // Return c_path too to keep it alive
}

// Helper to clean up
fn teardown_writer(handle: *mut FileWriterHandle) {
    if !handle.is_null() {
        let result = unsafe { file_writer_close(handle) };
        assert_eq!(result, FileWriterError::Success, "Failed to close writer");
    }
}

fn file_writer_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("FileWriter Performance");

    // --- Benchmark Raw Writes ---
    let sizes_to_test = [128, KIB, 64 * KIB, 1 * MIB];

    for size in sizes_to_test.iter().cloned() {
        // Prepare data once per size
        let data_to_write: Vec<u8> = vec![0xAB; size];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            format!("Write Raw {} Bytes", bytesize::ByteSize(size as u64)),
            &size,
            |b, &s| {
                // Use NamedTempFile to get a unique temporary path for each benchmark run
                let temp_file = NamedTempFile::new().expect("Failed to create temp file");
                let path = temp_file.path().to_str().expect("Path is not valid UTF-8");

                let (handle, _c_path) = setup_writer(path, FileWriterMode::Write);

                // Actual benchmark loop
                b.iter(|| {
                    let result = unsafe {
                        file_writer_write_raw(
                            handle,
                            black_box(data_to_write.as_ptr()), // black_box data pointer
                            black_box(s),                      // black_box size
                        )
                    };
                    // Don't assert inside the loop for performance, but check outside if needed
                    // assert_eq!(result, FileWriterError::Success);
                    // Preventing compiler optimization on the result if needed:
                    black_box(result);
                });

                teardown_writer(handle);
                // Temp file is automatically deleted when `temp_file` goes out of scope
            },
        );
    }

    // --- Benchmark String Writes (Example) ---
    let string_to_write = "This is a moderately long string for testing purposes.\n";
    let c_string = CString::new(string_to_write).unwrap();
    let string_len = string_to_write.len();

    group.throughput(Throughput::Bytes(string_len as u64));
    group.bench_function("Write String", |b| {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_str().expect("Path is not valid UTF-8");
        let (handle, _c_path) = setup_writer(path, FileWriterMode::Write);

        b.iter(|| {
            let result = unsafe {
                file_writer_write_string(
                    handle,
                    black_box(c_string.as_ptr()), // black_box string pointer
                )
            };
            black_box(result);
        });

        teardown_writer(handle);
    });

    // --- Benchmark BufWriter vs std::fs::File Directly (for comparison) ---
    // This shows the benefit of BufWriter
    let size_for_comparison = 1 * MIB;
    let data_for_comparison: Vec<u8> = vec![0xCD; size_for_comparison];

    group.throughput(Throughput::Bytes(size_for_comparison as u64));
    group.bench_function("Direct File Write 1 MiB", |b| {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path();
        let mut file = fs::File::create(path).expect("Failed to create raw file");

        b.iter(|| {
            file.write_all(black_box(&data_for_comparison))
                .expect("Raw write failed");
            // file.flush().expect("Raw flush failed"); // Flushing every iter is slow
        });
        file.flush().expect("Final raw flush failed"); // Flush once at the end
    });

    group.bench_function("BufWriter Write 1 MiB (Rust)", |b| {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path();
        let file = fs::File::create(path).expect("Failed to create buffered file");
        // Default 8k buffer
        let mut writer = std::io::BufWriter::new(file);

        b.iter(|| {
            writer
                .write_all(black_box(&data_for_comparison))
                .expect("Buffered write failed");
            // No flush needed per iteration due to buffering
        });
        writer.flush().expect("Final buffered flush failed"); // Flush once at the end
    });

    group.finish();
}

criterion_group!(benches, file_writer_benchmarks);
criterion_main!(benches);
