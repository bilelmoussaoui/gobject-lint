#include <glib.h>

void test_nested_if(void) {
    GError *error = NULL;

    if (g_file_test("test.txt", G_FILE_TEST_EXISTS)) {
        g_file_get_contents("test.txt", NULL, NULL, &error);

        if (error) {
            g_warning("Error: %s", error->message);
            g_error_free(error);
        }
    }
}
