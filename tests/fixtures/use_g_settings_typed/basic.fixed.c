#include <gio/gio.h>

static void
my_func (GSettings *settings)
{
  g_settings_set_string (settings, "name", "hello");
  g_settings_set_int (settings, "count", 42);

  int val = g_settings_get_int (settings, "count");
}
