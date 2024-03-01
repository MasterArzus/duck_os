    .section .text.entry
    .globl _start
_start:
    # hart_id is in a0, a0 --> t0
    # pc = 0x8020_0000

    # 1.set stack
    la sp, boot_stack_lower_bound
    # 2.set table & satp
    # satp: 8 << 60 | boot_pagetable
    la t0, boot_pagetable
    li t1, 8 << 60
    srli t0, t0, 12
    or t0, t0, t1
    csrw satp, t0
    sfence.vma
    # 3.reset stack
    # the offset is 0xffff_ffc0_0000_0000
    # TODO: 学习一下关于 addi 和 lui 的相关内容。为什么直接 addi会报错？？？？
    li t1, 0xffffffc000000000
    add sp, sp, t1
    # 4.jump to rust_main
    la t0, rust_main
    add t0, t0, t1
    jr t0


    .section .bss.stack

    .globl boot_stack_lower_bound
boot_stack_lower_bound:

    .space 4096 * 16 * 8  # 8 CPUS at most

    .globl boot_stack_top
boot_stack_top:

    .section .data
    .align 12
boot_pagetable:
    # two pte
    # 0x0000_0000_8000_0000 ---> 0x0000_0000_8000_0000
    # 0xffff_ffc0_8000_0000 ---> 0x0000_0000_8000_0000
    .quad 0
    .quad 0
    .quad (0x80000 << 10) | 0xcf #VRWXAD
    .zero 8 * 255
    .quad (0x80000 << 10) | 0xcf
    .zero 8 * 253

    