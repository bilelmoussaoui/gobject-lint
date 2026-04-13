#include <glib-object.h>

static void
foo_class_init (FooClass *klass)
{
  g_type_class_add_private (klass, sizeof (FooPrivate));
}
