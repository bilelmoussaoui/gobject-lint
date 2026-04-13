#include <glib.h>

static void
my_func (char *str, char *other, char *ptr)
{
  if (str)
    g_free (str);

  if (other)
    g_free (other);

  if (ptr != NULL)
    g_free (ptr);
}
