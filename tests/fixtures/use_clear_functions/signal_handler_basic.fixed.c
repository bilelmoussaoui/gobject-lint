#include <glib-object.h>

typedef struct {
  GObject *source;
  gulong   signal_id;
  gulong   notify_id;
} MyObj;

/* 2-statement pattern: disconnect then zero */

static void
clear_2stmt (MyObj *self)
{
  g_clear_signal_handler (&self->signal_id, self->source);
}

/* Multiple IDs in same function */

static void
clear_2stmt_multiple (MyObj *self)
{
  g_clear_signal_handler (&self->signal_id, self->source);
  g_clear_signal_handler (&self->notify_id, self->source);
}

/* if-guarded: if (id) — guard is redundant, replace entire if */

static void
clear_if_truthy (MyObj *self)
{
  g_clear_signal_handler (&self->signal_id, self->source);
}

/* if-guarded: if (id > 0) */

static void
clear_if_gt_zero (MyObj *self)
{
  g_clear_signal_handler (&self->signal_id, self->source);
}

/* if-guarded: if (id != 0) */

static void
clear_if_neq_zero (MyObj *self)
{
  g_clear_signal_handler (&self->notify_id, self->source);
}
