#include <glib.h>

static void
my_func (char *str, char *other, char *ptr, char **strv)
{
  if (str)
    g_free (str);

  if (other)
    g_free (other);

  if (ptr != NULL)
    g_free (ptr);

  if (strv)
    g_strfreev (strv);

  if (other != NULL)
    g_free_size (other, 128);
}
