#include <glib.h>

void test_freed(void) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        g_warning("Error: %s", error->message);
        g_error_free(error);
    }
}
