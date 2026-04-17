#include <glib.h>

gboolean test_propagate_in_else(GError **error_out) {
    GError *error = NULL;

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (!error) {
        return TRUE;
    } else {
        g_propagate_error(error_out, error);
        return FALSE;
    }
}
