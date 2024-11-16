pub fn command_fn(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let command: &[u8] = &(*buffer)[row][3..];

        let mut b = 0;
        for i in command.iter() {
            if *i != b'\0' {
                b += 1;
            }
        }

        b += 3;
        let comm: &[u8] = &(*buffer)[row][3..b];

        let commands: [Command; 5] = [
            Command::new("hello", hello_action),
            Command::new("error", error_action),
            Command::new("reboot", reboot_action),
            Command::new("shutdown", shutdown_action),
            Command::new("clear", clear),
        ];

        for cmd in commands.iter() {
            if comm == cmd.name.as_bytes() {
                let result = (cmd.action)(buffer, row);
                if result {
                    return true;
                }
                return false; // Завершите цикл, если команда найдена, но не вернула true
            }
        }

        error_action(buffer, row);
        false // Возвращаем false, если команда не найдена
    }
}

fn hello_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let msg = b"HELLO!";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }
        false
    }
}

fn error_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let msg = b"Error: command";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }
        false
    }
}

fn reboot_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let msg = b"Rebooting...";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }

        asm!(
            "cli",            // Отключаем прерывания
            "out 0x64, al",   // Отправляем команду на контроллер клавиатуры
            "2: hlt",         // Метка 2: останавливаем процессор
            "jmp 2b",         // Переход к метке 2, чтобы создать бесконечный цикл
            in("al") 0xFEu8   // Значение 0xFE для команды перезагрузки
        );
        false
    }
}

fn shutdown_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let msg = b"Shutting down...";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }

        asm!(
            "cli",            // Отключаем прерывания
            "mov ax, 0x5301", // Подключаемся к APM API
            "xor bx, bx",
            "int 0x15",
            "mov ax, 0x530E", // Устанавливаем версию APM на 1.2
            "xor bx, bx",
            "mov cx, 0x0102",
            "int 0x15",
            "mov ax, 0x5307", // Выключаем систему
            "mov bx, 0x0001",
            "mov cx, 0x0003",
            "int 0x15",
            "hlt", // Останавливаем процессор
            options(noreturn, nostack)
        );
        false
    }
}
