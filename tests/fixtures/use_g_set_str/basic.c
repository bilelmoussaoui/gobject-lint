#include <glib.h>

static void
my_func (char **name, const char *new_name)
{
  g_free (*name);
  *name = g_strdup (new_name);
}
