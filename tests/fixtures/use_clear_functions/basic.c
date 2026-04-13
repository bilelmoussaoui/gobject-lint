#include <glib.h>
#include <glib-object.h>

static void
my_func (GObject *obj, char *str)
{
  if (obj) {
    g_object_unref (obj);
    obj = NULL;
  }

  if (str) {
    g_free (str);
    str = NULL;
  }
}
