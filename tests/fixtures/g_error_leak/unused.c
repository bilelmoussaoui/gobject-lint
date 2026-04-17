#include <glib.h>

void test_unused(void) {
    GError *error = NULL;

    // Error is declared but never used - no leak
}
