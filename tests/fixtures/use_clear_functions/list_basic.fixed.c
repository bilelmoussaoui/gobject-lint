#include <glib.h>

static void
my_func (GList *list, GSList *slist)
{
  g_clear_list (&list, NULL);

  g_clear_list (&list, g_free);

  g_clear_slist (&slist, NULL);

  g_clear_slist (&slist, g_free);

  g_clear_list (&list, NULL);

  g_clear_list (&list, g_free);

  g_clear_slist (&slist, NULL);

  g_clear_slist (&slist, g_free);
}
