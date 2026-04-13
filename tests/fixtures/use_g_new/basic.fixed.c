#include <glib.h>

typedef struct { int x; } MyStruct;

static void
my_func (void)
{
  MyStruct *s = g_new (MyStruct, 1);
  MyStruct *z = g_new0 (MyStruct, 1);
}
