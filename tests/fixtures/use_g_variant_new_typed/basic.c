#include <glib.h>

static void
my_func (void)
{
  GVariant *v1 = g_variant_new ("s", "hello");
  GVariant *v2 = g_variant_new ("b", TRUE);
  GVariant *v3 = g_variant_new ("i", 42);
}
