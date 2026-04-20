#include <glib-object.h>

typedef struct {
  GObject *item;
} MyPrivate;

void
set_item (MyPrivate *priv, GObject *new_item)
{
  g_clear_object (&priv->item);
  priv->item = g_object_ref (new_item);
}
