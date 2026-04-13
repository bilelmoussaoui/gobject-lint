#include <glib-object.h>

static void
my_func (void)
{
  FooObject *obj = g_object_new (FOO_TYPE_OBJECT, NULL);
  g_object_set (obj, "name", "hello", NULL);
}
