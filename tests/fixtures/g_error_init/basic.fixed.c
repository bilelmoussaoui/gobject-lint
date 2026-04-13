#include <glib.h>

static void
my_func (void)
{
  GError *error = NULL;

  do_something (&error);
}

static void
already_ok (void)
{
  GError *error = NULL;

  do_something (&error);
}
