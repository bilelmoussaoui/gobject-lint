#include <glib.h>

void test_g_clear_error(void) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        g_warning("Error: %s", error->message);
        g_clear_error(&error);
    }
}
