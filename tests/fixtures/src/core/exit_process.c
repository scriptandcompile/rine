// Calls ExitProcess() directly instead of returning from main.
// Tests: kernel32 ExitProcess → std::process::exit.
#include <windows.h>
#include <stdio.h>

int main(void) {
    puts("before exit");
    ExitProcess(7);
    // Should never reach here.
    puts("FAIL: after ExitProcess");
    return 99;
}
