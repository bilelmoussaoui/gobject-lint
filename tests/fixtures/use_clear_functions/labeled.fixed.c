#include <glib-object.h>

typedef struct {
  GObject *source;
  gchar   *name;
  gulong   signal_id;
  guint    timeout_id;
} MyObj;

/* consecutive free+NULL after a goto label */

static int
labeled_consecutive (MyObj *self)
{
  self->name = g_strdup ("test");
  if (!self->name)
    goto fail;
  return 0;
fail:
  g_clear_pointer (&self->name, g_free);
  return -1;
}

/* signal disconnect+zero after a goto label */

static int
labeled_signal (MyObj *self)
{
  self->signal_id = g_signal_connect (self->source, "notify", NULL, NULL);
  if (!self->signal_id)
    goto cleanup;
  return 0;
cleanup:
  g_clear_signal_handler (&self->signal_id, self->source);
  return -1;
}

/* handle-id cleanup+zero after a goto label */

static int
labeled_handle_id (MyObj *self)
{
  self->timeout_id = g_timeout_add (100, NULL, NULL);
  if (!self->timeout_id)
    goto out;
  return 0;
out:
  g_clear_handle_id (&self->timeout_id, g_source_remove);
  return -1;
}

/* bare disconnect on member after a goto label */

static int
labeled_bare_disconnect (MyObj *self)
{
  self->signal_id = g_signal_connect (self->source, "notify", NULL, NULL);
  if (!self->signal_id)
    goto error;
  return 0;
error:
  g_clear_signal_handler (&self->signal_id, self->source);
  return -1;
}

/* stmt2 (NULL assignment) has its own label */

static int
labeled_stmt2_null (MyObj *self)
{
  self->name = g_strdup ("test");
  if (!self->name)
    goto fail;
  return 0;
fail:
  g_clear_pointer (&self->name, g_free);
  return -1;
}
