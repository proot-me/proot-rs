/**
 * This code is used to test whether the `CLONE_FS` flag in the clone() system
 * call is handled correctly by proot-rs.
 *
 * The program first spawns a child process with `clone(CLONE_FS)`, then calls
 * `chdir()` in the child process, and then calls `getcwd()` in both the child
 * and the parent process.
 *
 * The expected result is that the `cwd` of the child process and the `cwd` of
 * the parent process are always the same.
 */

#define _GNU_SOURCE

#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <unistd.h>

// Stack size for cloned child
#define STACK_SIZE (1024 * 1024)

static void exit_with_error(char *msg) {
    fprintf(stderr, "%s\n", msg);
    exit(1);
}

// Start function for cloned child
static int child_func() {
    char buf[1024];
    // Get the cwd value before modification, and print it.
    getcwd(buf, sizeof(buf));
    puts(buf);

    // Change cwd
    chdir("/etc");

    // Query the cwd value after modification, and print it.
    getcwd(buf, sizeof(buf));
    puts(buf);

    // We need to flush stdout manually otherwise in some cases (e.g. output to
    // pipe) the content will be lost.
    fflush(stdout);
    return 0;
}

// Start function for parent process
static int parent_func() {
    char buf[1024];
    // Query the value of cwd from the parent process and print it out
    getcwd(buf, sizeof(buf));
    puts(buf);
    return 0;
}

int main(int argc, char const *argv[]) {
    // Allocate stack area for child
    char *stack = mmap(NULL, STACK_SIZE, PROT_READ | PROT_WRITE,
                       MAP_PRIVATE | MAP_ANONYMOUS | MAP_STACK, -1, 0);
    if (stack == MAP_FAILED)
        exit_with_error("Error while mmap()");
    char *stack_top = stack + STACK_SIZE;

    // Create a child to execute some code.
    pid_t pid = clone(child_func, stack_top, CLONE_FS | SIGCHLD, NULL);
    if (pid == -1)
        exit_with_error("Error while clone()");

    // Wait for child to return from it's function
    if (waitpid(pid, NULL, 0) == -1)
        exit_with_error("Error while waitpid()");

    return parent_func();
}
