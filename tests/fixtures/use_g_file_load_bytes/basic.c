#include <gio/gio.h>

static void
my_func (GFile *file, GCancellable *cancellable, GError **error)
{
  char *contents = NULL;
  gsize length = 0;

  g_file_load_contents (file, cancellable, &contents, &length, NULL, error);
  GBytes *bytes = g_bytes_new_take (contents, length);
}
