#include "file_writer/file_writer.h"
#include <catch2/catch_test_macros.hpp>
#include <string>
#include <vector>
#include <fstream>
#include <iostream>
#include <cstdio>

std::string readFileContent(const std::string& filename) {
    std::ifstream ifs(filename, std::ios::binary);
    if (!ifs) {
        return "";
    }
    return std::string((std::istreambuf_iterator<char>(ifs)),
                       (std::istreambuf_iterator<char>()));
}

void cleanupFile(const std::string& filename) {
    std::remove(filename.c_str());
}

TEST_CASE("FileWriter Basic Operations", "[file_writer]") {
    const char* test_filename = "test_basic.txt";
    cleanupFile(test_filename);

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
        handle = nullptr;

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
        err = file_writer_new(test_filename, &handle, FileWriterMode::Write);
        REQUIRE(err == FileWriterError::Success);
        const char* first_part = "Part1;";
        err = file_writer_write_string(handle, first_part);
        REQUIRE(err == FileWriterError::Success);
        err = file_writer_close(handle);
        REQUIRE(err == FileWriterError::Success);
        handle = nullptr;

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

        err = file_writer_flush(handle);
        REQUIRE(err == FileWriterError::Success);


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

        err = file_writer_set_buffer_size(handle, 16);
        REQUIRE(err == FileWriterError::Success);

        std::string long_message(100, 'X');
        err = file_writer_write_string(handle, long_message.c_str());
        REQUIRE(err == FileWriterError::Success);

        err = file_writer_set_buffer_size(handle, 8192);
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
        REQUIRE(err == FileWriterError::InvalidHandle);

        err = file_writer_flush(invalid_handle);
        REQUIRE(err == FileWriterError::InvalidHandle);

        err = file_writer_close(invalid_handle);
        REQUIRE(err == FileWriterError::InvalidHandle);
    }


    if (handle != nullptr) {
        file_writer_close(handle);
    }
    cleanupFile(test_filename);
}