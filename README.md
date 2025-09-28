# crusty
A simple C compiler which targets x86-64 gcc 15.2

# Usage
crusty will automatically assemble and link the generated assembly for you.
```
$ crusty main.c
```

# Features & Info
- Local variable declarations.
- Function declarations.
- Function calls with argument passing.
- Variadic function support, such as `printf` (which depends on libc).
- `char*`, `char`, `int` support.
- System V ABI calling convention on x86-64 Linux.

```c
int main() {
    char* msg = "Hello, World!";
    printf(msg);
}
```

```
.section .rodata
.LC0:
    .string "Hello, World!
"
.section .text
    .globl main
    .type main, @function
main:
    pushq %rbp
    movq %rsp, %rbp
    leaq .LC0(%rip), %rax
    movq %rax, -8(%rbp)
    movq -8(%rbp), %rdi
    movl $0, %eax
    call printf
    leave
    ret

```
