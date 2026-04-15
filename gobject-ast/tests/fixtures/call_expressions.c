void test_function(void) {
    GTask *task = g_task_new(NULL, NULL, NULL, NULL);
    g_task_set_source_tag(task, test_function);
    g_object_unref(task);
}
