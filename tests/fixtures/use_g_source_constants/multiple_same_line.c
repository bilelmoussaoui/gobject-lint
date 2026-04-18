#include <glib.h>

/* Callback that has multiple return statements on same line (conditional) */
static gboolean
my_callback (gpointer data)
{
  int *counter = data;
  (*counter)++;

  /* Multiple TRUE/FALSE on same line */
  return (*counter < 10) ? TRUE : FALSE;
}

static void
test_setup (void)
{
  int *counter = g_new0 (int, 1);
  g_idle_add (my_callback, counter);
}
