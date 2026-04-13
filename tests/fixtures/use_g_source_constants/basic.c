#include <glib.h>

static gboolean
my_timeout_cb (gpointer user_data)
{
  g_print ("tick\n");
  return TRUE;
}

static gboolean
my_idle_cb (gpointer user_data)
{
  return FALSE;
}

static void
setup (void)
{
  g_timeout_add (1000, my_timeout_cb, NULL);
  g_idle_add (my_idle_cb, NULL);
}
