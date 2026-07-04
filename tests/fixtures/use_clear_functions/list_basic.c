#include <glib.h>

static void
my_func (GList *list, GSList *slist)
{
  g_list_free (list);
  list = NULL;

  g_list_free_full (list, g_free);
  list = NULL;

  g_slist_free (slist);
  slist = NULL;

  g_slist_free_full (slist, g_free);
  slist = NULL;

  if (list != NULL) {
    g_list_free (list);
    list = NULL;
  }

  if (list != NULL) {
    g_list_free_full (list, g_free);
    list = NULL;
  }

  if (slist != NULL) {
    g_slist_free (slist);
    slist = NULL;
  }

  if (slist != NULL) {
    g_slist_free_full (slist, g_free);
    slist = NULL;
  }
}
