.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
# TODO：暂时没有考虑中断嵌套的问题，如果 U -> S -> S 那会发生什么？
__alltraps:
    # 不能使用spp来判断，因为此时没有空余的寄存器可以使用
    # 用户态：则交换sp sscratch； 内核态：不用交换
    bgtz sp, __user_to_kernel

__trap_entry:
    # sp -> kernel stack
    addi sp, sp, -35*8
    sd x1, 1*8(sp)
    .set n, 3
    .rept 29
        SAVE_GP %n
        .set n, n+1
    .endr
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    csrr t2, sscratch
    sd t2, 2*8(sp)
    ld tp, 34*8(sp)
    mv a0, sp
    call trap_handler

__restore:
    # sp -> kernel stack, sscratch -> user stack
    # restore sstatus/sepc/user_stack_sp
    ld t0, 32*8(sp) # t0 <- sstatus
    ld t1, 33*8(sp) # t1 <- sepc
    ld t2, 2*8(sp)  # t2 <- sp
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    
    # get SPP 在第8位
    # 之后要恢复t0，所以这里要先判断，后恢复
    andi t0, t0, 0x100
    bnez t0, __kernel_to_kernel

    ld x1, 1*8(sp)
    # skip x2(sp)，已保存
    .set n, 3
    .rept 29
        LOAD_GP %n
        .set n, n+1
    .endr
    addi sp, sp, 35
    csrrw sp, sscratch, sp
    beqz sp, __idle
    sret

__kernel_to_kernel:
    # 内核中断时，恢复现场，不用恢复fld，不用交换寄存器
    ld x1, 1*8(sp)
    # skip x2(sp)，已保存
    .set n, 3
    .rept 29
        LOAD_GP %n
        .set n, n+1
    .endr
    addi sp, sp, 35
    sret

__user_to_kernel:
    csrrw sp, sscratch, sp
    j __trap_entry