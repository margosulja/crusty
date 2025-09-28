.section .rodata
.LC0:
    .string "hello"
.LC1:
    .string "%d, %s, %d\n"
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
