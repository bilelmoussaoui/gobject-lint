#include <glib.h>
#include <string.h>

static void
my_func (const char *a, const char *b)
{
  if (strcmp (a, b) == 0)
    g_print ("equal\n");

  if (strcmp (a, b) != 0)
    g_print ("not equal\n");
}
