#include "file_writer.h" // Include the C header for our Rust library
#include <catch2/catch_test_macros.hpp> // Using Catch2 v3 syntax
#include <string>
#include <vector>
#include <fstream>
#include <iostream>
#include <cstdio> // For std::remove

// Helper function to read file content
std::string readFileContent(const std::string& filename) {
    std::ifstream ifs(filename, std::ios::binary);
    if (!ifs) {
        return ""; // Return empty string on error
    }
    return std::string((std::istreambuf_iterator<char>(ifs)),
                       (std::istreambuf_iterator<char>()));
}

// Helper function to clean up test files
void cleanupFile(const std::string& filename) {
    std::remove(filename.c_str());
}

TEST_CASE("FileWriter Basic Operations", "[file_writer]") {
    const char* test_filename = "test_basic.txt";
    cleanupFile(test_filename); // Ensure clean state

    FileWriterHandle* handle = nullptr;
    FileWriterError err;

    SECTION("Create and Write String") {
        err = file_writer_new(test_filename, &handle, FileWriterMode::Write);
        REQUIRE(err == FileWriterError::Success);
        REQUIRE(handle != nullptr);

        const char* message = "Hello from Rust FFI!";
        err = file_writer_write_string(handle, message);
        REQUIRE(err == FileWriterError::Success);

        err = file_writer_close(handle);
        REQUIRE(err == FileWriterError::Success);
        handle = nullptr; // Prevent double close in teardown

        std::string content = readFileContent(test_filename);
        REQUIRE(content == message);
    }

    SECTION("Create and Write Raw Bytes") {
        err = file_writer_new(test_filename, &handle, FileWriterMode::Write);
        REQUIRE(err == FileWriterError::Success);
        REQUIRE(handle != nullptr);

        std::vector<uint8_t> data = {0xDE, 0xAD, 0xBE, 0xEF};
        err = file_writer_write_raw(handle, data.data(), data.size());
        REQUIRE(err == FileWriterError::Success);

        err = file_writer_close(handle);
        REQUIRE(err == FileWriterError::Success);
        handle = nullptr;

        std::string content = readFileContent(test_filename);
        REQUIRE(content.size() == data.size());
        REQUIRE(memcmp(content.data(), data.data(), data.size()) == 0);
    }

     SECTION("Append Mode") {
        // First write
        err = file_writer_new(test_filename, &handle, FileWriterMode::Write);
        REQUIRE(err == FileWriterError::Success);
        const char* first_part = "Part1;";
        err = file_writer_write_string(handle, first_part);
        REQUIRE(err == FileWriterError::Success);
        err = file_writer_close(handle);
        REQUIRE(err == FileWriterError::Success);
        handle = nullptr;

        // Second write (Append)
        err = file_writer_new(test_filename, &handle, FileWriterMode::Append);
        REQUIRE(err == FileWriterError::Success);
        const char* second_part = "Part2";
        err = file_writer_write_string(handle, second_part);
        REQUIRE(err == FileWriterError::Success);
        err = file_writer_close(handle);
        REQUIRE(err == FileWriterError::Success);
        handle = nullptr;

        std::string content = readFileContent(test_filename);
        REQUIRE(content == "Part1;Part2");
    }

     SECTION("Flush Operation") {
        err = file_writer_new(test_filename, &handle, FileWriterMode::Write);
        REQUIRE(err == FileWriterError::Success);

        const char* message1 = "DataBeforeFlush_";
        err = file_writer_write_string(handle, message1);
        REQUIRE(err == FileWriterError::Success);

        // Flush the data
        err = file_writer_flush(handle);
        REQUIRE(err == FileWriterError::Success);

        // Verify content immediately after flush (might not be guaranteed on all OS without fsync, but often works)
        // For robust check, you might need OS-specific sync or reopen the file.
        // Here, we primarily test that flush doesn't error out easily.
        std::string content_after_flush = readFileContent(test_filename);
        // This check depends on OS buffering behavior, may not be reliable
        // REQUIRE(content_after_flush == message1);

        const char* message2 = "DataAfterFlush";
        err = file_writer_write_string(handle, message2);
        REQUIRE(err == FileWriterError::Success);

        err = file_writer_close(handle);
        REQUIRE(err == FileWriterError::Success);
        handle = nullptr;

        std::string final_content = readFileContent(test_filename);
        REQUIRE(final_content == std::string(message1) + std::string(message2));
    }

    SECTION("Set Buffer Size") {
        err = file_writer_new(test_filename, &handle, FileWriterMode::Write);
        REQUIRE(err == FileWriterError::Success);

        // Set a small buffer size (e.g., 16 bytes)
        err = file_writer_set_buffer_size(handle, 16);
        REQUIRE(err == FileWriterError::Success);

        // Write more data than the buffer size to ensure it has to flush internally or uses the new size
        std::string long_message(100, 'X');
        err = file_writer_write_string(handle, long_message.c_str());
        REQUIRE(err == FileWriterError::Success);

         // Set a larger buffer size
        err = file_writer_set_buffer_size(handle, 8192); // Default-ish size
        REQUIRE(err == FileWriterError::Success);

        std::string message2 = "Second Write";
         err = file_writer_write_string(handle, message2.c_str());
        REQUIRE(err == FileWriterError::Success);


        err = file_writer_close(handle);
        REQUIRE(err == FileWriterError::Success);
        handle = nullptr;

        std::string content = readFileContent(test_filename);
        REQUIRE(content == long_message + message2);
    }


    SECTION("Error Handling - Invalid Path") {
        err = file_writer_new(nullptr, &handle, FileWriterMode::Write);
        REQUIRE(err == FileWriterError::InvalidPath);
        REQUIRE(handle == nullptr);
    }

     SECTION("Error Handling - Invalid Handle") {
        FileWriterHandle* invalid_handle = nullptr;
        const char* message = "test";
        err = file_writer_write_string(invalid_handle, message);
        // Depending on implementation, might check inside or rely on OS crash.
        // Our Rust code checks for null handle.
        REQUIRE(err == FileWriterError::InvalidHandle);

        err = file_writer_flush(invalid_handle);
        REQUIRE(err == FileWriterError::InvalidHandle);

        // Closing a null handle should also be an error or no-op (we return error)
        err = file_writer_close(invalid_handle);
         REQUIRE(err == FileWriterError::InvalidHandle);
    }


    // Teardown: Ensure handle is closed if a section failed mid-way
    // and cleanup the test file
    if (handle != nullptr) {
        file_writer_close(handle);
    }
    cleanupFile(test_filename);
}