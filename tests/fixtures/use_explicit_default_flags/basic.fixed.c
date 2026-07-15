#include <gio/gio.h>
#include <glib-object.h>

static void
my_func (void)
{
  GApplication *app = g_application_new ("com.example.App", G_APPLICATION_DEFAULT_FLAGS);
  GFile *file = g_file_new_for_path ("/tmp/foo");
  GFileInfo *info = g_file_query_info (file, "*", G_FILE_QUERY_INFO_NONE, NULL, NULL);
  g_signal_connect_object (app, "activate", G_CALLBACK (g_application_quit), NULL, G_CONNECT_DEFAULT);
}
