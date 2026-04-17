#include <glib.h>

void test_multiple_calls(void) {
    GError *error = NULL;

    g_file_get_contents("test1.txt", NULL, NULL, &error);
    if (error) {
        g_error_free(error);
        error = NULL;
    }

    g_file_get_contents("test2.txt", NULL, NULL, &error);
    // Error from second call is leaked!
}
