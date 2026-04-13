#include <glib.h>
#include <glib-object.h>

static void
my_func (GObject *obj, char *str)
{
  g_clear_object (&obj);

  g_clear_pointer (&str, g_free);
}
