#include <glib.h>
#include <string.h>

static gboolean
test_multiple_same_line (const char *str1, const char *str2)
{
  /* Multiple strncmp prefix checks on same line */
  if (g_str_has_prefix (str1, "http://") && g_str_has_prefix (str2, "https://"))
    return TRUE;

  /* Multiple strcmp suffix checks on same line */
  if (g_str_has_suffix (str1, ".txt") || g_str_has_suffix (str2, ".md"))
    return TRUE;

  return FALSE;
}
