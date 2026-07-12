#include <glib.h>

static gboolean
my_idle_cb (gpointer user_data)
{
  do_work ();
  return G_SOURCE_REMOVE;
}

static gboolean
my_timeout_cb (gpointer user_data)
{
  do_work ();
  return G_SOURCE_REMOVE;
}

static gboolean
my_timeout_seconds_cb (gpointer user_data)
{
  do_work ();
  return G_SOURCE_REMOVE;
}

static void
setup (void)
{
  g_idle_add (my_idle_cb, NULL);

  guint id = g_timeout_add (10, my_timeout_cb, NULL);

  gpointer pid = GUINT_TO_POINTER (g_timeout_add_seconds (1, my_timeout_seconds_cb, NULL));
}
