#include <glib.h>
#include <string.h>

static void
my_func (const char *a, const char *b)
{
  if (g_str_equal (a, b))
    g_print ("equal\n");

  if (!g_str_equal (a, b))
    g_print ("not equal\n");
}
