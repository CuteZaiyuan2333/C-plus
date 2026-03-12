struct resource {
    int id;
} bind resource {
    resource(i) {
        host.id = i;
        printf("Resource %d created\n", host.id);
    }
    void destroy() {
        printf("Resource %d destroyed\n", host.id);
    }
}

int main() {
    {
        let resource r.resource(1);
        printf("Inside scope\n");
    }
    printf("Outside scope\n");
    return 0;
}
