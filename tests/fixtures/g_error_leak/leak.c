#include <glib.h>

void test_leak(void) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    // Error is not freed or propagated - this is a leak!
}
