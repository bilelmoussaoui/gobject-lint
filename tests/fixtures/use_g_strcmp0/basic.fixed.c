#include <glib.h>

static gboolean
my_func (const char *a,
         const char *b)
{
  if (g_strcmp0 (a, b) == 0)
    return TRUE;

  if (strncmp (a, b, 3) == 0)
    return TRUE;

  return FALSE;
}
