#include <glib-object.h>

static void
my_func (GValue *value)
{
  g_value_set_string (value, "hello");
}
