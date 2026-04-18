#include <glib.h>
#include <string.h>

static gboolean
test_multiple_same_line (const char *str1, const char *str2)
{
  /* Multiple strncmp prefix checks on same line */
  if (strncmp (str1, "http://", strlen ("http://")) == 0 && strncmp (str2, "https://", strlen ("https://")) == 0)
    return TRUE;

  /* Multiple strcmp suffix checks on same line */
  if (strcmp (str1 + strlen (str1) - strlen (".txt"), ".txt") == 0 || strcmp (str2 + strlen (str2) - strlen (".md"), ".md") == 0)
    return TRUE;

  return FALSE;
}
