#include <glib.h>

static void
timed_exit_cb (GMainLoop *loop)
{
  g_main_loop_quit (loop);
}

void foo (GMainLoop *main_loop, gboolean do_timed_exit)
{
  if (do_timed_exit) {
    g_timeout_add_seconds_once (30, (GSourceOnceFunc) timed_exit_cb, main_loop);
  }
}
