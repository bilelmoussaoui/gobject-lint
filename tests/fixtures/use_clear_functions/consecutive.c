#include <glib.h>

typedef struct {
  gchar *display_seat_id;
  gchar *name;
  gchar *path;
} MyObj;

void test_function(MyObj *self) {
    g_free (self->display_seat_id);
    self->display_seat_id = NULL;
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
    g_free (self->name);
    self->name = NULL;
    g_free (self->path);
    self->path = NULL;
    return -1;
}
