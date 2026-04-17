#include <glib.h>
#include <gio/gio.h>

void async_operation(GCancellable *cancellable,
                      GAsyncReadyCallback callback,
                      gpointer user_data) {
    GError *error = NULL;
    g_autoptr(GTask) task = NULL;

    task = g_task_new(NULL, cancellable, callback, user_data);

    g_file_get_contents("test.txt", NULL, NULL, &error);

    if (error) {
        // g_task_return_error takes ownership of the error - not a leak
        g_task_return_error(task, error);
        return;
    }

    g_task_return_boolean(task, TRUE);
}
