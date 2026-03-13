#include <stdio.h>
#include <string.h>
#include "compiler.h"
#include "builder.h"
int main() {
    let Compiler compiler.Compiler(1);
    compiler.compile_file(100);
    let Builder builder.Builder(".", 1);
    builder.build_project();
    return 0;
}
