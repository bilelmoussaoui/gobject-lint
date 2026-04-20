#include <glib-object.h>

void
set_object_with_unref (GObject **obj, GObject *new_obj)
{
  g_object_unref (*obj);
  *obj = g_object_ref (new_obj);
}
