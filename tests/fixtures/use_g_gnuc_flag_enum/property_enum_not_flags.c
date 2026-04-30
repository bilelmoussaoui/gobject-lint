// Should NOT trigger: property enum, not a flags enum
// Even though PROP_FIRST = 1 (power of 2), this is clearly a property enum
typedef enum {
  PROP_FIRST = 1,
  PROP_SECOND,
  PROP_THIRD,
  PROP_FOURTH
} MyProperties;
