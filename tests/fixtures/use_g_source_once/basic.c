#include <glib.h>

static gboolean
my_idle_cb (gpointer user_data)
{
  do_work ();
  return G_SOURCE_REMOVE;
}

static void
setup (void)
{
  g_idle_add (my_idle_cb, NULL);
}
