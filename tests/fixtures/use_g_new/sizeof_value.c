#include <glib.h>

typedef struct {
  int len;
  int data[1];
} MyBitmask;

static MyBitmask *
my_bitmask_copy (MyBitmask *src)
{
  MyBitmask *mask;

  mask = g_malloc (sizeof (*src));

  return mask;
}

static void *
alloc_value (void)
{
  int val;

  return g_malloc (sizeof (val));
}

typedef struct { int x; } ctx_t;

static ctx_t *
alloc_ctx (void)
{
  ctx_t val;

  ctx_t *p = g_malloc (sizeof (ctx_t));
  void *q = g_malloc (sizeof (val));

  return p;
}

struct NoTypedef {
  int y;
};

static struct NoTypedef *
alloc_no_typedef (void)
{
  struct NoTypedef val;
  struct NoTypedef *ptr;

  struct NoTypedef *p = g_malloc (sizeof (struct NoTypedef));
  void *q = g_malloc (sizeof (val));
  void *r = g_malloc (sizeof (*ptr));

  return p;
}

static void
alloc_autofree_struct (void)
{
  g_autofree struct NoTypedef *info = NULL;

  info = g_malloc (sizeof (*info));
}

static void *
alloc_double_ptr (void)
{
  char **ptr;

  return g_malloc (sizeof (*ptr));
}

typedef struct { int val; } ext_type_t;

static void *
alloc_ext_type (void)
{
  return g_malloc (sizeof (ext_type_t));
}

#define BUFFER_SIZE 4096

static void *
alloc_macro_sizeof_no_warn (void)
{
  return g_malloc (sizeof (BUFFER_SIZE));
}
