struct resource {
    int id;
} bind resource {
    resource(i) {
        host.id = i;
        printf("Resource %d created\n", host.id);
    }
    void destroy() {
        printf("Resource %d destroyed in thread\n", host.id);
    }
}

void task(let resource r) {
    printf("Task processing resource %d\n", r.id);
}

int main() {
    let resource r.resource(101);
    printf("Spawning thread...\n");
    spawn task(r); 
    
    // 错误尝试：r 在 spawn 后已被移动，不能再次访问
    printf("Illegal access: %d\n", r.id);
    
    return 0;
}
