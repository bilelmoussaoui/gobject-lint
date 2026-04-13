#include <glib-object.h>

static void
foo_dispose (GObject *object)
{
  FooPrivate *priv = foo_get_instance_private (FOO (object));
  g_clear_object (&priv->child);
}
