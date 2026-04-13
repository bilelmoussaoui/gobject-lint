#include <gio/gio.h>

static void
my_func (GSettings *settings)
{
  g_settings_set_value (settings, "name", g_variant_new ("s", "hello"));
  g_settings_set_value (settings, "count", g_variant_new ("i", 42));

  int val = g_variant_get_int32 (g_settings_get_value (settings, "count"));
}
