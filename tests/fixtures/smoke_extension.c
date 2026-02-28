#ifdef _WIN32
#define SQLITE_EXTENSION_EXPORT __declspec(dllexport)
#else
#define SQLITE_EXTENSION_EXPORT
#endif

SQLITE_EXTENSION_EXPORT int sqlite3_extension_init(void *db, char **pz_err_msg, void *p_api) {
    (void)db;
    (void)pz_err_msg;
    (void)p_api;
    return 0; // SQLITE_OK
}

SQLITE_EXTENSION_EXPORT int sqlite3_smokeext_init(void *db, char **pz_err_msg, void *p_api) {
    (void)db;
    (void)pz_err_msg;
    (void)p_api;
    return 0; // SQLITE_OK
}
