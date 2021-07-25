/**
 * This program is designed to test the handling of fork() and vfork() and
 * clone() functions. Not only test the call of those functions, this program
 * will combine them, and test the correctness of these functions when called
 * in nesting.
 * We define the nested calls of these three functions as a series of
 * actions: ACTION_FORK(1), ACTION_VFORK(2), ACTION_CLONE(3).
 * The test is performed by generating a sequence of a certain length, and the
 * length is the depth of the iteration. which means that a total of
 * 3^DEPTH_OF_FORK tests will be executed.
 * In each test, we call one of these three functions in the sequence defined
 * previously in nesting. For example, for a sequence [1, 2, 3], this program
 * call fork() to create a child, and then the child will call vfork() to
 * generate another new child, which will then call clone(). To observe the
 * results, each child process will print out the id of the current action, and
 * we can test whether it passes by checking the program output.
 */

#define _GNU_SOURCE

#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#define ACTION_EMPTY 0
#define ACTION_FORK 1
#define ACTION_VFORK 2
#define ACTION_CLONE 3

#define ACTION_BITS_LEN 2
#define ACTION_BITS_MASK ((1 << ACTION_BITS_LEN) - 1)

#define STACK_SIZE (1024 * 1024)

#define DEPTH_OF_FORK 3

static void exit_with_error(char *msg) {
    fprintf(stderr, "%s\n", msg);
    exit(1);
}

void do_things(size_t action) { printf("%d", action); }

void wait_for_child_exit(pid_t pid) {
    while (1) {
        int wstatus = 0;
        waitpid(pid, &wstatus, 0);
        if (WIFEXITED(wstatus)) {
            if (WEXITSTATUS(wstatus) != 0) {
                char buf[1024];
                sprintf(buf, "child process %d terminated with status %d", pid,
                        WEXITSTATUS(wstatus));
                exit_with_error(buf);
            }
            return;
        } else if (WIFSIGNALED(wstatus)) {
            char buf[1024];
            sprintf(buf, "child process %d terminated by signal %d", pid,
                    WTERMSIG(wstatus));
            exit_with_error(buf);
            return;
        }
    }
}

int clone_child_func(size_t actions) {
    do_things(ACTION_CLONE);
    perform(actions >> ACTION_BITS_LEN);
    return 0;
}

void perform(size_t actions) {
    size_t action = actions & ACTION_BITS_MASK;
    if (action == ACTION_EMPTY) {
    } else if (action == ACTION_FORK) {
        pid_t pid = fork();
        if (pid == 0) { // child
            do_things(ACTION_FORK);
            perform(actions >> ACTION_BITS_LEN);
            _exit(0);
        } else if (pid == -1) {
            exit_with_error("Error while fork()");
        } else { // parent
            wait_for_child_exit(pid);
        }
    } else if (action == ACTION_VFORK) {
        pid_t pid = vfork();
        if (pid == 0) { // child
            do_things(ACTION_VFORK);
            perform(actions >> ACTION_BITS_LEN);
            _exit(0);
        } else if (pid == -1) {
            exit_with_error("Error while vfork()");
        } else { // parent
            wait_for_child_exit(pid);
        }
    } else if (action == ACTION_CLONE) {
        char *stack = mmap(NULL, STACK_SIZE, PROT_READ | PROT_WRITE,
                           MAP_PRIVATE | MAP_ANONYMOUS | MAP_STACK, -1, 0);
        if (stack == MAP_FAILED)
            exit_with_error("Error while mmap()");
        char *stack_top = stack + STACK_SIZE;

        pid_t pid =
            clone(clone_child_func, stack_top, CLONE_FS | SIGCHLD, actions);
        if (pid == -1)
            exit_with_error("Error while clone()");

        wait_for_child_exit(pid);
    }
}

void build_and_perform(int depth, size_t actions) {
    if (depth == 0) {
        perform(actions);
        printf(" ");
    } else {
        build_and_perform(depth - 1,
                          (actions << ACTION_BITS_LEN) | ACTION_FORK);
        build_and_perform(depth - 1,
                          (actions << ACTION_BITS_LEN) | ACTION_VFORK);
        build_and_perform(depth - 1,
                          (actions << ACTION_BITS_LEN) | ACTION_CLONE);
    }
}

int main(int argc, char const *argv[]) {
    setvbuf(stdout, NULL, _IONBF, 0);
    build_and_perform(DEPTH_OF_FORK, 0);
    return 0;
}
