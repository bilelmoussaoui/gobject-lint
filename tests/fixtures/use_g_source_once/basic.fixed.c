#include <glib.h>

static void
my_idle_cb (gpointer user_data)
{
  do_work ();
}

static void
setup (void)
{
  g_idle_add_once (my_idle_cb, NULL);
}
