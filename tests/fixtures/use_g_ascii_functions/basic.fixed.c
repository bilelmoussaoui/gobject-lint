#include <glib.h>
#include <ctype.h>

static void
my_func (char c, const char *str)
{
  /* These are all locale-dependent — should use g_ascii_* */
  char lower = g_ascii_tolower (c);
  char upper = g_ascii_toupper (c);

  if (g_ascii_isdigit (c))
    lower = c;

  if (g_ascii_isalpha (c))
    upper = c;

  if (g_ascii_isalnum (c) || g_ascii_isspace (c))
    lower = c;

  if (g_ascii_isupper (c) || g_ascii_islower (c))
    upper = c;

  /* These are fine — not in our list */
  g_print ("%c %c %s\n", lower, upper, str);
}
