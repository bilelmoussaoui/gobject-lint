void test_function(void) {
    const char *data = g_bytes_get_data(bytes, &size);
    g_bytes_unref(bytes);
}
