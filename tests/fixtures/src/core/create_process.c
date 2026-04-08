// Tests the CreateProcessA API call.
// This test verifies that CreateProcessA can spawn a child process successfully.
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    STARTUPINFOA si;
    PROCESS_INFORMATION pi;
    BOOL result;
    
    // Initialize the STARTUPINFO structure
    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);
    
    // Initialize the PROCESS_INFORMATION structure
    ZeroMemory(&pi, sizeof(pi));
    
    // Test creating a process that should succeed
    // Using the current executable as the target
    char cmd_line[] = "exit_code.exe";
    result = CreateProcessA(
        NULL,           // Application name (use command line)
        cmd_line,       // Command line
        NULL,           // Process handle not inheritable
        NULL,           // Thread handle not inheritable
        FALSE,          // Set handle inheritance to FALSE
        0,              // No creation flags
        NULL,           // Use parent's environment
        NULL,           // Use parent's starting directory
        &si,            // Pointer to STARTUPINFO structure
        &pi             // Pointer to PROCESS_INFORMATION structure
    );
    
    if (result) {
        puts("CreateProcessA ok\n");
        // Close process and thread handles
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
        return 0;
    } else {
        puts("CreateProcessA failed\n");
        return 1;
    }
}