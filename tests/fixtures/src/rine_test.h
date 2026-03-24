// Minimal output helpers for rine integration tests.
// Avoids printf/fprintf which pull in locale machinery that rine doesn't
// yet support. Uses puts() and a simple itoa for integer output.

#ifndef RINE_TEST_H
#define RINE_TEST_H

#include <stdio.h>
#include <string.h>

// Print an integer to stdout followed by newline, using puts.
static void put_int(int n) {
    char buf[32];
    int neg = 0;
    int i = 0;

    if (n < 0) {
        neg = 1;
        // avoid overflow on INT_MIN by treating the first digit specially
        if (n == -2147483647 - 1) {
            puts("-2147483648");
            return;
        }
        n = -n;
    }
    if (n == 0) {
        buf[i++] = '0';
    } else {
        while (n > 0) {
            buf[i++] = '0' + (n % 10);
            n /= 10;
        }
    }
    if (neg) buf[i++] = '-';

    // reverse
    for (int j = 0; j < i / 2; j++) {
        char tmp = buf[j];
        buf[j] = buf[i - 1 - j];
        buf[i - 1 - j] = tmp;
    }
    buf[i] = '\0';
    puts(buf);
}

// Print "label: value\n" without printf.
static void put_kv_int(const char *label, int value) {
    char buf[128];
    char num[32];
    int neg = 0;
    int i = 0;
    int n = value;

    if (n < 0) {
        neg = 1;
        if (n == -2147483647 - 1) {
            // handle INT_MIN
            strcpy(num, "-2147483648");
            i = 11;
        } else {
            n = -n;
        }
    }
    if (i == 0) {
        if (n == 0) {
            num[i++] = '0';
        } else {
            while (n > 0) {
                num[i++] = '0' + (n % 10);
                n /= 10;
            }
        }
        if (neg) num[i++] = '-';
        // reverse
        for (int j = 0; j < i / 2; j++) {
            char tmp = num[j];
            num[j] = num[i - 1 - j];
            num[i - 1 - j] = tmp;
        }
    }
    num[i] = '\0';

    // Build "label: value"
    int len = strlen(label);
    if (len + 2 + i >= (int)sizeof(buf)) {
        puts("FAIL: put_kv_int overflow");
        return;
    }
    memcpy(buf, label, len);
    buf[len] = ':';
    buf[len + 1] = ' ';
    memcpy(buf + len + 2, num, i + 1);
    puts(buf);
}

// Print "label: str\n" without printf.
static void put_kv_str(const char *label, const char *value) {
    char buf[256];
    int llen = strlen(label);
    int vlen = strlen(value);
    if (llen + 2 + vlen >= (int)sizeof(buf)) {
        puts("FAIL: put_kv_str overflow");
        return;
    }
    memcpy(buf, label, llen);
    buf[llen] = ':';
    buf[llen + 1] = ' ';
    memcpy(buf + llen + 2, value, vlen + 1);
    puts(buf);
}

#endif // RINE_TEST_H
