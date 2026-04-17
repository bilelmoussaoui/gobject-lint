#include <glib.h>

void test_double_pointer(GError **error_out) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        // Just set the out parameter - this is propagation
        *error_out = error;
    }
}
