#include <glib-object.h>

struct _MyWidget {
    GtkWidget parent_instance;
};

static void my_editable_init (GtkEditableInterface *iface);
static void my_scrollable_init (GtkScrollableInterface *iface);

G_DEFINE_TYPE_WITH_CODE (MyWidget, my_widget, GTK_TYPE_WIDGET,
                         G_ADD_PRIVATE (MyWidget)
                         G_IMPLEMENT_INTERFACE (GTK_TYPE_EDITABLE, my_editable_init)
                         G_IMPLEMENT_INTERFACE (GTK_TYPE_SCROLLABLE, my_scrollable_init))

static void
my_editable_init (GtkEditableInterface *iface)
{
}

static void
my_scrollable_init (GtkScrollableInterface *iface)
{
}

static void
my_widget_class_init (MyWidgetClass *klass)
{
}

static void
my_widget_init (MyWidget *self)
{
}
