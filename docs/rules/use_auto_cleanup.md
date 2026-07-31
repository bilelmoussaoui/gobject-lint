This rule detects manual cleanup patterns that can be replaced with `g_autoptr` for automatic resource management.

## Why?

- **Safety**: Prevents memory leaks from early returns or error paths
- **Cleaner code**: No manual cleanup needed
- **Compiler support**: Works with GCC cleanup attribute

## Examples

**Bad** (manual cleanup):
```c
void
my_function (void)
{
  GObject *obj = g_object_new (MY_TYPE_OBJECT, NULL);
  
  // ... use obj ...
  
  g_object_unref (obj);
}
```

**Good** (automatic cleanup):
```c
void
my_function (void)
{
  g_autoptr(GObject) obj = g_object_new (MY_TYPE_OBJECT, NULL);
  
  // ... use obj ...
  // Automatic cleanup when function returns!
}
```

## Options

### `allocation_proof` (default: `true`)

When `true`, the rule only suggests auto-cleanup when the variable is provably
allocated in the current function (e.g. via `g_object_new`, `g_strdup`,
`g_new0`, etc.). This avoids false positives for variables that are freed but
not locally allocated (borrowed pointers, out-parameters, etc.).

Set to `false` to flag any manually freed variable, regardless of whether the
allocation is visible:

```toml
[rules.use_auto_cleanup]
allocation_proof = false
```
