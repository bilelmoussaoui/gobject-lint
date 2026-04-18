#include <glib.h>
#include <ctype.h>

static gboolean
test_multiple_same_line (char a, char b, char c)
{
  /* Multiple tolower on same line */
  if (tolower (a) == tolower (b))
    return TRUE;

  /* Multiple isdigit on same line */
  if (isdigit (a) && isdigit (b) && isdigit (c))
    return TRUE;

  /* Mixed: tolower and toupper on same line */
  return tolower (a) == 'a' || toupper (b) == 'B';
}
