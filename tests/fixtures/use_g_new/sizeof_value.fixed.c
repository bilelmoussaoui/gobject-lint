#include <glib.h>

typedef struct {
  int len;
  int data[1];
} MyBitmask;

static MyBitmask *
my_bitmask_copy (MyBitmask *src)
{
  MyBitmask *mask;

  mask = g_new (MyBitmask, 1);

  return mask;
}

static void *
alloc_value (void)
{
  int val;

  return g_new (int, 1);
}

typedef struct { int x; } ctx_t;

static ctx_t *
alloc_ctx (void)
{
  ctx_t val;

  ctx_t *p = g_new (ctx_t, 1);
  void *q = g_new (ctx_t, 1);

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

  struct NoTypedef *p = g_new (struct NoTypedef, 1);
  void *q = g_new (struct NoTypedef, 1);
  void *r = g_new (struct NoTypedef, 1);

  return p;
}

static void
alloc_autofree_struct (void)
{
  g_autofree struct NoTypedef *info = NULL;

  info = g_new (struct NoTypedef, 1);
}

static void *
alloc_double_ptr (void)
{
  char **ptr;

  return g_new (char *, 1);
}

typedef struct { int val; } ext_type_t;

static void *
alloc_ext_type (void)
{
  return g_new (ext_type_t, 1);
}

#define BUFFER_SIZE 4096

static void *
alloc_macro_sizeof_no_warn (void)
{
  return g_malloc (sizeof (BUFFER_SIZE));
}
