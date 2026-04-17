#include <glib.h>

void test_not_initialized(void) {
    GError *error;

    // Error not initialized to NULL - should be caught by g_error_init rule, not this one
}
