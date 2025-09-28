# crusty
A simple C compiler which targets x86-64 gcc 15.2

Code generation is dead simple and can only support these straight forward compilations, more support will be added soon.

# Features & Info
• Local variable declarations.

• Function declarations.

• `char*`, `char`, `int` support.

• System V ABI calling convention on x86-64 Linux.

```c
int main() {
    int x = 42;
}
```

```asm
.section .text
    .globl main
    .type main, @function
main:
    pushq %rbp
    movq %rsp, %rbp
    movl $42, -4(%rbp)
    movl $0, %eax
    popq %rbp
    ret
```