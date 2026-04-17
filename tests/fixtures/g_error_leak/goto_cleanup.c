#include <glib.h>

gboolean test_goto_cleanup(void) {
    GError *error = NULL;
    gboolean success = FALSE;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        goto cleanup;
    }

    success = TRUE;

cleanup:
    g_clear_error(&error);
    return success;
}
