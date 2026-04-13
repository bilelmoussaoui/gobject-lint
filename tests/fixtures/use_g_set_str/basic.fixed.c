#include <glib.h>

static void
my_func (char **name, const char *new_name)
{
  g_set_str (&*name, new_name);
}
