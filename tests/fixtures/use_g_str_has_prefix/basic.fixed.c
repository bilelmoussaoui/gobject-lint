#include <glib.h>
#include <string.h>

static gboolean
my_func (const char *str)
{
  /* Prefix check — should use g_str_has_prefix */
  if (g_str_has_prefix (str, "foo"))
    return TRUE;

  /* Negated prefix check */
  if (!g_str_has_prefix (str, "bar"))
    return FALSE;

  /* Reversed operands: 0 == strncmp(...) */
  if (g_str_has_prefix (str, "baz"))
    return TRUE;

  /* strncmp with a numeric length — not detected, too risky */
  if (strncmp (str, "qux", 3) == 0)
    return FALSE;

  /* strcmp == 0 — handled by use_g_str_equal, not us */
  if (strcmp (str, "quux") == 0)
    return FALSE;

  return TRUE;
}
