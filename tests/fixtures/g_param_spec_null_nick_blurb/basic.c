#include <glib-object.h>

static void
foo_class_init (FooClass *klass)
{
  GObjectClass *object_class = G_OBJECT_CLASS (klass);

  g_object_class_install_property (object_class, PROP_NAME,
    g_param_spec_string ("name", "Name", "The name", NULL, G_PARAM_READWRITE));
}
