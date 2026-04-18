#include <glib.h>
#include <ctype.h>

static gboolean
test_multiple_same_line (char a, char b, char c)
{
  /* Multiple tolower on same line */
  if (g_ascii_tolower (a) == g_ascii_tolower (b))
    return TRUE;

  /* Multiple isdigit on same line */
  if (g_ascii_isdigit (a) && g_ascii_isdigit (b) && g_ascii_isdigit (c))
    return TRUE;

  /* Mixed: tolower and toupper on same line */
  return g_ascii_tolower (a) == 'a' || g_ascii_toupper (b) == 'B';
}
