// CP-TUI: A simple TUI library test for C+ capabilities
// Standard C headers are included by the transpiler automatically

struct Terminal {
    int _initialized;
}

bind Terminal {
    Terminal() {
        host._initialized = 1;
        printf("\x1b[?25l"); // Hide cursor
        printf("\x1b[2J");   // Clear screen
        printf("\x1b[H");    // Move cursor to (0, 0)
        printf("--- CP-TUI System Initialized ---\n");
    }
    
    void destroy() {
        printf("\x1b[?25h"); // Show cursor
        printf("\n[CP-TUI] Terminal state restored.\n");
    }
    
    void move_to(int x, int y) {
        printf("\x1b[%d;%dH", y, x);
    }
    
    void print_at(int x, int y, char* text) {
        printf("\x1b[%d;%dH%s", y, x, text);
    }
}

struct Component {
    int x;
    int y;
}

bind Component {
    Component(x, y) {
        host.x = x;
        host.y = y;
    }
    void draw() {
        printf("\x1b[%d;%dH(Generic Component)", host.y, host.x);
    }
}

// Use fork to create a Label which is a component with text
fork Component as Label {
    + char* text;
} bind {
    Label(int x, int y, char* text) {
        host.x = x;
        host.y = y;
        host.text = text;
    }
    void draw() {
        printf("\x1b[%d;%dH%s", host.y, host.x, host.text);
    }
}

// Use fork to create a Bordered Panel
fork Component as Panel {
    + int w;
    + int h;
} bind {
    Panel(int x, int y, int w, int h) {
        host.x = x;
        host.y = y;
        host.w = w;
        host.h = h;
    }
    void draw() {
        // Draw top border
        printf("\x1b[%d;%dH+", host.y, host.x);
        for(int i=0; i < host.w-2; i++) printf("-");
        printf("+");
        
        // Draw sides
        for(int j=1; j < host.h-1; j++) {
            printf("\x1b[%d;%dH|", host.y + j, host.x);
            printf("\x1b[%d;%dH|", host.y + j, host.x + host.w - 1);
        }
        
        // Draw bottom border
        printf("\x1b[%d;%dH+", host.y + host.h - 1, host.x);
        for(int i=0; i < host.w-2; i++) printf("-");
        printf("+");
    }
}

int main() {
    // RAII Terminal management
    let Terminal term.Terminal();
    
    term.move_to(5, 3);
    printf("Welcome to C+ TUI Demonstration!");

    // Create components
    let Panel p.Panel(5, 5, 40, 10);
    p.draw();
    
    let Label l1.Label(8, 7, "Ownership: Managed");
    let Label l2.Label(8, 8, "Structural Evolution: Forked");
    let Label l3.Label(8, 9, "Memory Safety: RAII Guaranteed");
    
    l1.draw();
    l2.draw();
    l3.draw();
    
    term.move_to(1, 16);
    printf("Press enter to exit...");
    getchar();
    
    // When main returns, 'term' is destroyed, restoring terminal state.
    return 0;
}