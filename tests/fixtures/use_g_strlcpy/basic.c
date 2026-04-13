#include <glib.h>
#include <string.h>

static void
my_func (const char *src)
{
  char buf[64];

  /* Unsafe — no bounds checking */
  strcpy (buf, src);

  /* Unsafe — no bounds checking */
  strcat (buf, src);

  /* Unsafe — strncat's n is max bytes to append, not buffer size */
  strncat (buf, src, sizeof (buf) - strlen (buf) - 1);

  g_print ("%s\n", buf);
}
