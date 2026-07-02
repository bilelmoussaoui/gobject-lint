#include <glib.h>

typedef struct {
  gchar *display_seat_id;
  gchar *name;
  gchar *path;
} MyObj;

void test_function(MyObj *self) {
    g_clear_pointer (&self->display_seat_id, g_free);
}

int test_labeled(MyObj *self) {
    self->name = g_strdup ("hello");
    if (!self->name)
        goto fail;
    self->path = g_strdup ("world");
    if (!self->path)
        goto fail;
    return 0;
fail:
    g_clear_pointer (&self->name, g_free);
    g_clear_pointer (&self->path, g_free);
    return -1;
}
