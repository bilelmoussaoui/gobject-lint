#include "basic.h"

/* Single-line G_DEFINE without semicolon */
G_DEFINE_FINAL_TYPE (KioskApp, kiosk_app, G_TYPE_OBJECT);

/* Multi-line G_DEFINE_FINAL_TYPE_WITH_CODE without semicolon */
G_DEFINE_FINAL_TYPE_WITH_CODE (KioskAreaConstraint, kiosk_area_constraint, G_TYPE_OBJECT,
                               G_IMPLEMENT_INTERFACE (META_TYPE_EXTERNAL_CONSTRAINT,
                                                      kiosk_area_constraint_iface_init));

/* Already correct - has semicolon */
G_DEFINE_TYPE (CorrectType, correct_type, G_TYPE_OBJECT);

/* Multi-line G_DEFINE_TYPE without semicolon */
G_DEFINE_TYPE_WITH_PRIVATE (MyType,
                            my_type,
                            G_TYPE_OBJECT);
