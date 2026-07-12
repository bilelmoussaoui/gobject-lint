#include <glib.h>

static void
my_idle_cb (gpointer user_data)
{
  do_work ();
}

static void
my_timeout_cb (gpointer user_data)
{
  do_work ();
}

static void
my_timeout_seconds_cb (gpointer user_data)
{
  do_work ();
}

static void
setup (void)
{
  g_idle_add_once (my_idle_cb, NULL);

  guint id = g_timeout_add_once (10, my_timeout_cb, NULL);

  gpointer pid = GUINT_TO_POINTER (g_timeout_add_seconds_once (1, my_timeout_seconds_cb, NULL));
}
