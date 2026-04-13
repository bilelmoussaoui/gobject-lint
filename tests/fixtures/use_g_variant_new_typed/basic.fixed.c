#include <glib.h>

static void
my_func (void)
{
  GVariant *v1 = g_variant_new_string ("hello");
  GVariant *v2 = g_variant_new_boolean (TRUE);
  GVariant *v3 = g_variant_new_int32 (42);
}
