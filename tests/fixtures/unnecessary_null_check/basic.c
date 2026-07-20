#include <glib.h>

static void
my_func (char *str, char *other, char *ptr, char **strv, GList *list)
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

  if (list->data != NULL)
    g_free (list->data);

  if (*strv != NULL)
    g_free (*strv);

  if (G_UNLIKELY (ptr != NULL))
    g_free (ptr);
}
