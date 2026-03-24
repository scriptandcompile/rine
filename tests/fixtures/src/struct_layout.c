// Tests struct layout and pointer arithmetic.
// Tests: data alignment, memory layout correctness.
#include "rine_test.h"

struct Point {
    int x;
    int y;
};

struct Rect {
    struct Point top_left;
    struct Point bottom_right;
};

int main(void) {
    struct Rect r = {{1, 2}, {3, 4}};
    int area = (r.bottom_right.x - r.top_left.x) *
               (r.bottom_right.y - r.top_left.y);
    put_kv_int("area", area);
    put_kv_int("sizeof_rect", (int)sizeof(struct Rect));
    return 0;
}
