#include <glib.h>
#include <ctype.h>

static void
my_func (char c, const char *str)
{
  /* These are all locale-dependent — should use g_ascii_* */
  char lower = tolower (c);
  char upper = toupper (c);

  if (isdigit (c))
    lower = c;

  if (isalpha (c))
    upper = c;

  if (isalnum (c) || isspace (c))
    lower = c;

  if (isupper (c) || islower (c))
    upper = c;

  /* These are fine — not in our list */
  g_print ("%c %c %s\n", lower, upper, str);
}
