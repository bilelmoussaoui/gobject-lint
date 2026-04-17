#include <glib.h>

gboolean test_early_return(void) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        g_warning("Error: %s", error->message);
        g_error_free(error);
        return FALSE;
    }

    return TRUE;
}
