// Tests FindClose family of Windows API calls.
// Tests: FindFirstFileA, FindNextFileA, FindClose
#include <windows.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    // Create a test file first
    HANDLE hFile = CreateFileA("test_file.txt", GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL);
    if (hFile == INVALID_HANDLE_VALUE) {
        puts("FAIL: Could not create test file");
        return 1;
    }
    
    // Write some data to the test file
    DWORD bytes_written;
    const char* test_data = "test data for find operations";
    WriteFile(hFile, test_data, strlen(test_data), &bytes_written, NULL);
    CloseHandle(hFile);
    
    // Use FindFirstFileA to find the test file
    WIN32_FIND_DATAA find_data;
    HANDLE hFind = FindFirstFileA("test_file.txt", &find_data);
    
    if (hFind == INVALID_HANDLE_VALUE) {
        puts("FAIL: FindFirstFileA failed");
        return 1;
    }
    
    // Verify we found the correct file
    if (strcmp(find_data.cFileName, "test_file.txt") != 0) {
        puts("FAIL: Found wrong file");
        FindClose(hFind);
        return 1;
    }
    
    // Test FindClose - should succeed
    if (!FindClose(hFind)) {
        puts("FAIL: FindClose failed");
        return 1;
    }
    
    if (!DeleteFileA("test_file.txt")) {
        puts("FAIL: DeleteFileA failed");
        return 1;
    }

    // Test FindClose with invalid handle
    if (FindClose(INVALID_HANDLE_VALUE)) {
        puts("FAIL: FindClose should fail with invalid handle");
        return 1;
    }
    
    puts("FindClose works");
    return 0;
}