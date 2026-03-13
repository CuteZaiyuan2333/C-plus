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
    // 子线程结束时应自动调用 resource_destroy
}

int main() {
    let resource r.resource(101);
    printf("Spawning thread...\n");
    spawn task(r); 
    
    // 给子线程一点运行时间
    #include <unistd.h>
    sleep(1);
    
    printf("Main thread exit\n");
    return 0;
}
