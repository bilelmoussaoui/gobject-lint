#include <glib-object.h>

typedef struct {
  GObject *item;
} MyPrivate;

void
set_item (MyPrivate *priv, GObject *new_item)
{
  g_set_object (&priv->item, new_item);
}
