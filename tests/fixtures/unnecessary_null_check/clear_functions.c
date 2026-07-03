#include <glib.h>

// Valid: Actually uses g_clear_list, so NULL check is unnecessary
void
test_clear_list (void)
{
  GList *list = NULL;

  // ... some code ...

  if (list) {
    g_clear_list (&list, g_free);
  }
}
