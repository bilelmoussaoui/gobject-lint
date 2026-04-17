#include <glib.h>

void test_multiple_errors(void) {
    GError *error1 = NULL;
    GError *error2 = NULL;

    g_file_get_contents("test1.txt", NULL, NULL, &error1);
    g_file_get_contents("test2.txt", NULL, NULL, &error2);

    // error1 is freed, error2 is leaked
    if (error1) {
        g_error_free(error1);
    }
}
