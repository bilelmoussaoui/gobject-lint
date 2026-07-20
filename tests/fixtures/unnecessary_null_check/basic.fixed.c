#include <glib.h>

static void
my_func (char *str, char *other, char *ptr, char **strv, GList *list)
{
  g_free (str);

  g_free (other);

  g_free (ptr);

  g_strfreev (strv);

  g_free_size (other, 128);

  g_free (list->data);

  g_free (*strv);

  g_free (ptr);
}
