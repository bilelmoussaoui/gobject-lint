#include <glib.h>

static char *
my_func (void)
{
  GString *str = g_string_new ("hello");
  g_string_append (str, " world");
  return g_string_free_and_steal (str);
}
