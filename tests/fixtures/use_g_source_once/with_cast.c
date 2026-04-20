#include <glib.h>

static gboolean
timed_exit_cb (GMainLoop *loop)
{
  g_main_loop_quit (loop);
  return FALSE;
}

void foo (GMainLoop *main_loop, gboolean do_timed_exit)
{
  if (do_timed_exit) {
    g_timeout_add_seconds (30, (GSourceFunc) timed_exit_cb, main_loop);
  }
}
