#include <glib.h>

typedef struct {
  char *name;
} MyObj;

/* s1 unlabeled, s2 labeled NULL assignment */

static int
labeled_null_assign (MyObj *self, char *name)
{
  self->name = name;
done:
  name = NULL;
  return 0;
}

/* s1 labeled assignment, s2 unlabeled NULL assignment */

static int
labeled_s1_assign (MyObj *self, char *name)
{
done:
  self->name = name;
  name = NULL;
  return 0;
}
