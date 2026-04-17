#include <glib.h>

gboolean test_goto_leak(void) {
    GError *error = NULL;
    gboolean success = FALSE;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        goto cleanup;
    }

    success = TRUE;

cleanup:
    // Error is not freed here - leak!
    return success;
}
