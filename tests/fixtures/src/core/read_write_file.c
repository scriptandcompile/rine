// Tests both WriteFile and ReadFile API calls.
// Tests: CreateFile, WriteFile, ReadFile, SetFilePointer, CloseHandle.
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    // Create a temporary file
    char file_path[] = "read_write_test_file.txt";
    HANDLE hFile = CreateFileA(
        file_path,
        GENERIC_READ | GENERIC_WRITE,
        0,                    // No sharing
        NULL,                 // Default security
        CREATE_ALWAYS,        // Create new file
        FILE_ATTRIBUTE_NORMAL,
        NULL                  // No template
    );
    
    if (hFile == INVALID_HANDLE_VALUE) {
        puts("CreateFile failed");
        return 1;
    }

    puts("CreateFile ok");
    
    // Write some data to the file
    const char write_data[] = "Hello, ReadFile test!";
    DWORD bytes_written;
    BOOL write_result = WriteFile(hFile, write_data, sizeof(write_data) - 1, &bytes_written, NULL);
    
    if (!write_result) {
        puts("WriteFile failed");
        CloseHandle(hFile);
        return 1;
    }

    puts("WriteFile ok");
    
    // Reset file pointer to beginning
    DWORD new_pos = SetFilePointer(hFile, 0, NULL, FILE_BEGIN);
    if (new_pos == INVALID_SET_FILE_POINTER) {
        puts("SetFilePointer failed");
        CloseHandle(hFile);
        return 1;
    }

    puts("SetFilePointer ok");
    
    // Read the data back
    char read_buffer[100];
    DWORD bytes_read;
    BOOL read_result = ReadFile(hFile, read_buffer, sizeof(read_buffer) - 1, &bytes_read, NULL);
    
    if (!read_result) {
        puts("ReadFile failed");
        CloseHandle(hFile);
        return 1;
    }
    
    // Null terminate the buffer for string comparison
    read_buffer[bytes_read] = '\0';
    
    // Verify the data matches
    if (strcmp(read_buffer, write_data) == 0) {
        puts("ReadFile ok");
    } else {
        puts("ReadFile data mismatch");
        puts(read_buffer);
        puts(write_data);
        CloseHandle(hFile);
        return 1;
    }

    // Clean up
    CloseHandle(hFile);
    DeleteFileA(file_path);
    
    return 0;
}