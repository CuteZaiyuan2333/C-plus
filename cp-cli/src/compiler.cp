struct Compiler {
    int verbose;
}
bind Compiler {
    Compiler(v) {
        host.verbose = v;
    }
    void compile_file(int id) {
        printf("Compiling unit %d\n", id);
    }
    void destroy() {
        printf("Compiler shutdown\n");
    }
}
