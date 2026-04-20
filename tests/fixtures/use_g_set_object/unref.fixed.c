#include <glib-object.h>

void
set_object_with_unref (GObject **obj, GObject *new_obj)
{
  g_set_object (&*obj, new_obj);
}
