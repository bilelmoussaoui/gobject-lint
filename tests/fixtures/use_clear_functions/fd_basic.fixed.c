#include <glib.h>
#include <unistd.h>

typedef struct {
  int fd;
  int sock;
} MyObj;

/* consecutive pair */

static void
close_consecutive (MyObj *self)
{
  g_clear_fd (&self->fd, NULL);
}

/* if-guarded: fd >= 0 */

static void
close_if_gte_zero (MyObj *self)
{
  g_clear_fd (&self->fd, NULL);
}

/* if-guarded: fd != -1 */

static void
close_if_neq_neg1 (MyObj *self)
{
  g_clear_fd (&self->sock, NULL);
}

/* g_close variant */

static void
close_g_close (MyObj *self)
{
  g_clear_fd (&self->fd, NULL);
}

/* g_close with error parameter */

static void
close_g_close_with_error (MyObj *self, GError **error)
{
  g_clear_fd (&self->fd, error);
}

/* not a match: assigned to 0 instead of -1 */

static void
close_wrong_sentinel (MyObj *self)
{
  close (self->fd);
  self->fd = 0;
}

/* not a match: if-guarded with wrong sentinel (0 instead of -1) */

static void
close_if_wrong_sentinel (MyObj *self)
{
  if (self->fd >= 0) {
    close (self->fd);
    self->fd = 0;
  }
}

/* not a match: pointer-style guard on fd pattern */

static void
close_if_null_guard (MyObj *self)
{
  if (self->fd != 0) {
    g_clear_fd (&self->fd, NULL);
  }
}
