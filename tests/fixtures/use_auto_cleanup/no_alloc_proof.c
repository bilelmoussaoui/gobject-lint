#include <glib-object.h>

gchar      **get_some_strv   (void);
gchar       *get_some_name   (void);
GObject     *get_some_object (void);
void         use_object      (GObject *obj);

/* GStrv from opaque source, freed locally — flagged when allocation_proof=false */
static void
strv_opaque_source (void)
{
  gchar **tokens = NULL;

  tokens = get_some_strv ();
  g_print ("%s\n", tokens[0]);
  g_strfreev (tokens);
}

/* g_free'd pointer from opaque source — flagged when allocation_proof=false */
static void
autofree_opaque_source (void)
{
  gchar *name = NULL;

  name = get_some_name ();
  g_print ("%s\n", name);
  g_free (name);
}

/* g_object_unref'd pointer from opaque source — flagged when allocation_proof=false */
static void
autoptr_opaque_source (void)
{
  GObject *obj = NULL;

  obj = get_some_object ();
  use_object (obj);
  g_object_unref (obj);
}

/* returned variable should NOT be flagged even with allocation_proof=false */
static gchar *
returned_opaque (void)
{
  gchar *name = NULL;

  name = get_some_name ();
  return name;
}
