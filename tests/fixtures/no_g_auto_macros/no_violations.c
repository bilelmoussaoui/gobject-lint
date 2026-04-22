#include <glib.h>

static void
test_function (void)
{
        char *str = g_strdup ("hello");
        guint8 *data = g_malloc (10);

        g_print ("%s\n", str);

        g_free (str);
        g_free (data);
}
