#include <glib-object.h>

void
set_object_manually (GObject **obj_ptr, GObject *new_obj)
{
  g_clear_object (obj_ptr);
  *obj_ptr = g_object_ref (new_obj);
}
