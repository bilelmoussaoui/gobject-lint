#include <glib-object.h>
#include <glib/gstdio.h>

static void
test_autofree (void)
{
  g_autofree char *str = NULL;

  do_something (&str);
}

static void
test_autoptr (void)
{
  g_autoptr(GList) list = NULL;

  do_something (&list);
}

static void
test_autofd (void)
{
  g_autofd int fd = -1;

  do_something (&fd);
}

static void
test_auto_queue (void)
{
  g_auto(GQueue) queue = G_QUEUE_INIT;

  do_something (&queue);
}

static void
test_auto_value (void)
{
  g_auto(GValue) value = G_VALUE_INIT;

  do_something (&value);
}

static void
already_ok (void)
{
  g_autofree char *str = NULL;
  g_autoptr(GList) list = NULL;
  g_autofd int fd = -1;
  g_auto(GQueue) queue = G_QUEUE_INIT;
  g_auto(GValue) value = G_VALUE_INIT;

  do_something (&str);
  do_something (&list);
  do_something (&fd);
  do_something (&queue);
  do_something (&value);
}

static void
test_nested_only (void)
{
  g_autofree char *str = NULL;

  if (TRUE) {
    str = g_strdup ("hello");
    do_something (str);
  }
}

static void
test_autolist (void)
{
  g_autolist(GList) list = NULL;

  do_something (&list);
}

static void
test_never_used (void)
{
  g_autofree char *str = NULL;
}
