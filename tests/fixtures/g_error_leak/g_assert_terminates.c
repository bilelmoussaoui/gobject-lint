#include <glib.h>

void test_g_assert_terminates(void) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    // g_assert(FALSE) terminates the program
    g_assert(!error);
}
