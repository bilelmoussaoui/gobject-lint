#include <glib.h>

typedef struct {
  GError *error;
} MyData;

/* GError ** pointing at an existing GError* field — already has a non-NULL
 * initializer, so we must NOT insert = NULL (would produce invalid C:
 * GError **error = NULL = &d->error). */

static void
my_func (MyData *d)
{
  GError **error = &d->error;

  do_something (error);
}

/* Same with a plain GError* assigned from a field — already initialized. */

static void
my_func2 (MyData *d)
{
  GError *error = d->error;

  do_something (&error);
}
