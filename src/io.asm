section .text
global inb

inb:
    mov dx, [rsp + 8]
    in al, dx
    ret
