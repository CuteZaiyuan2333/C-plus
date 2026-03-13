struct Builder {
    char* project_root;
    int is_debug;
}
bind Builder {
    Builder(root, debug) {
        host.project_root = root;
        host.is_debug = debug;
    }
    void build_project() {
        printf("Building %s in %d mode\n", host.project_root, host.is_debug);
    }
    void destroy() {
        printf("Builder done\n");
    }
}
