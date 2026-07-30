#include <glib-object.h>

static void
multi_autofree (const char *input)
{
  gchar *aaa = NULL, *bbb = NULL, *ccc = NULL;

  aaa = g_strdup (input);
  bbb = g_strdup ("hello");
  ccc = g_strdup ("world");

  g_print ("%s %s %s\n", aaa, bbb, ccc);
  g_free (aaa);
  g_free (bbb);
  g_free (ccc);
}

static void
multi_mixed (void)
{
  GObject *obj1 = NULL, *obj2 = NULL;

  obj1 = g_object_new (G_TYPE_OBJECT, NULL);
  obj2 = g_object_new (G_TYPE_OBJECT, NULL);
  use_objects (obj1, obj2);
  g_object_unref (obj1);
  g_object_unref (obj2);
}

static void
multi_partial (const char *input)
{
  gchar *allocated = NULL, *not_allocated = NULL;

  allocated = g_strdup (input);
  g_free (allocated);
  g_free (not_allocated);
}
