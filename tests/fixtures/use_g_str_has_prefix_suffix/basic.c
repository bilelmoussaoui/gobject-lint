#include <glib.h>
#include <string.h>

static gboolean
my_func (const char *str)
{
  /* Prefix check — should use g_str_has_prefix */
  if (strncmp (str, "foo", strlen ("foo")) == 0)
    return TRUE;

  /* Negated prefix check */
  if (strncmp (str, "bar", strlen ("bar")) != 0)
    return FALSE;

  /* Reversed operands: 0 == strncmp(...) */
  if (0 == strncmp (str, "baz", strlen ("baz")))
    return TRUE;

  /* strncmp with a numeric length — not detected, too risky */
  if (strncmp (str, "qux", 3) == 0)
    return FALSE;

  /* strcmp == 0 — handled by use_g_str_equal, not us */
  if (strcmp (str, "quux") == 0)
    return FALSE;

  /* Suffix check — should use g_str_has_suffix */
  if (strcmp (str + strlen (str) - strlen ("_bar"), "_bar") == 0)
    return TRUE;

  /* Negated suffix check */
  if (strcmp (str + strlen (str) - strlen (".baz"), ".baz") != 0)
    return FALSE;

  return TRUE;
}
