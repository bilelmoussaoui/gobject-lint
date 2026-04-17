#include <glib.h>

gboolean test_propagated(GError **error_out) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        g_propagate_error(error_out, error);
        return FALSE;
    }

    return TRUE;
}
