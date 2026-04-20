#include <glib-object.h>

void
set_object_manually (GObject **obj_ptr, GObject *new_obj)
{
  g_set_object (obj_ptr, new_obj);
}
