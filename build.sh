#!/bin/sh
set -e

# Создаем ISO-образ
mkdir -p iso/boot/grub
cp kernel.bin iso/boot/kernel.bin

# Файл конфигурации GRUB
cat > iso/boot/grub/grub.cfg <<EOF
menuentry "my_os" {
    multiboot /boot/kernel.bin
    boot
}
EOF

grub-mkrescue -o my_os.iso iso
