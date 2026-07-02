#include <glib.h>

static void
my_func (char *str, char *other, char *ptr, char **strv)
{
  g_free (str);

  g_free (other);

  g_free (ptr);

  g_strfreev (strv);

  g_free_size (other, 128);
}
