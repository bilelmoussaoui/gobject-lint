#include <glib-object.h>
#include <glib/gstdio.h>

/* Already initialized */
static void
initialized (void)
{
  g_autofree char *str = NULL;
  g_autoptr(GList) list = NULL;
  g_autofd int fd = -1;
  g_auto(GQueue) queue = G_QUEUE_INIT;
  g_auto(GValue) value = G_VALUE_INIT;

  str = g_strdup ("hello");
  list = g_list_append (NULL, NULL);
  fd = open ("/dev/null", 0);
}

/* Initialized with a value */
static void
initialized_with_value (void)
{
  g_autofree char *str = g_strdup ("hello");
  g_autoptr(GList) list = g_list_append (NULL, NULL);
  g_autofd int fd = open ("/dev/null", 0);

  do_something (str, list, fd);
}

/* First use is a direct assignment */
static void
direct_assignment (void)
{
  g_autofree char *str;
  g_autoptr(GList) list;
  g_autofd int fd;
  g_auto(GQueue) queue;

  str = g_strdup ("hello");
  list = g_list_append (NULL, NULL);
  fd = open ("/dev/null", 0);
  queue = (GQueue) G_QUEUE_INIT;
}

/* g_auto with unknown type — no known init macro, skip */
static void
unknown_auto_type (void)
{
  g_auto(GMutex) mutex;

  g_mutex_init (&mutex);
}
