struct Resource {
    int id = 0;
}

bind Resource {
    Resource(id) {
        host.id = id;
    }
    void destroy() {
        printf("Destroying Resource %d\n", host.id);
    }
}

fork Resource as EnhancedResource {
    + int extra = 99;
} bind {
    patch Resource(id) as EnhancedResource(id, ex) {
        host.id = id;
        host.extra = ex;
    }
    + void show() {
        printf("Enhanced: %d, %d\n", host.id, host.extra);
    }
}

int main() {
    let Resource r.Resource(1);
    if (1) {
        let Resource r2 = r; // moved
        printf("In if block\n");
    }
    // r.id; // Checker should error if we uncomment this

    let EnhancedResource er.EnhancedResource(2, 50);
    er.show();

    alias EnhancedResource a = er;
    a.show();

    return 0;
}
