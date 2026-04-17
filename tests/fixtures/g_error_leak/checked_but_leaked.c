#include <glib.h>

void test_checked_but_leaked(void) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        g_warning("Error: %s", error->message);
        // Oops! Forgot to free the error
    }
}
