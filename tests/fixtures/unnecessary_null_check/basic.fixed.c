#include <glib.h>

static void
my_func (char *str, char *other, char *ptr)
{
  g_free (str);

  g_free (other);

  g_free (ptr);
}
