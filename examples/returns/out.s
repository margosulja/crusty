.section .rodata
.LC0:
    .string "the value i got from the generous function: %d\n"
.section .text
    .globl main
    .type main, @function
give_value_please:
    pushq %rbp
    movq %rsp, %rbp
    movl $42, %eax
    leave
    ret
main:
    pushq %rbp
    movq %rsp, %rbp
    call give_value_please
    movl %eax, -8(%rbp)
    leaq .LC0(%rip), %rdi
    movl -8(%rbp), %esi
    movl $0, %eax
    call printf
    leave
    ret
