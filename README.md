# crusty
A simple C compiler which targets x86-64 gcc 15.2 linux

# Usage
crusty will automatically assemble and link the generated assembly for you.
```
$ crusty main.c
```

Check `examples` for some examples!

# Features & Info
- Local variable declarations with proper stack alignment.
- Function declarations with parameter support.
- Function calls with argument passing (up to 6 register arguments).
- User defined functions with proper parameter handling.
- Return statement support with proper return value handling.
- Function call assignment (storing return values in variables).
- Variadic function support, such as `printf` (which depends on libc).
- Type-specific register allocation (`char*` → 64-bit, `int` → 32-bit, `char` → 8-bit).
- String literal management with automatic `.rodata` section generation.
- Position Independent Executable (PIE) compatible code generation.
- Proper stack frame management with 16-byte alignment.
- Memory-safe variable storage with no stack overlaps.
- `char*`, `char`, `int` data type support.
- System V ABI calling convention on x86-64 Linux.
- Automatic GCC compilation and linking.

```c
int main() {
    int x = 1234;
    char* y = "hello";
    char z = 'z';
    printf("%d, %s, %c\n", x, y, z);
}
```

```
.section .rodata
.LC0:
    .string "hello"
.LC1:
    .string "%d, %s, %c\n"
.section .text
    .globl main
    .type main, @function
main:
    pushq %rbp
    movq %rsp, %rbp
    movl $1234, -8(%rbp)
    leaq .LC0(%rip), %rax
    movq %rax, -16(%rbp)
    movl $122, -24(%rbp)
    leaq .LC1(%rip), %rdi
    movl -8(%rbp), %esi
    movq -16(%rbp), %rdx
    movzbl -24(%rbp), %ecx
    movl $0, %eax
    call printf
    leave
    ret

```
