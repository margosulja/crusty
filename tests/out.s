.section .rodata
.LC0:
    .string "hello world!
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
