#include <glib.h>

void test_g_error_terminates(void) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        // g_error() terminates the program, so no leak
        g_error("Failed to read file: %s", error->message);
    }
}
