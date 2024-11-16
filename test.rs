pub fn command_fn(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let command: &[u8] = &(*buffer)[row][3..];

        // Найти конец команды, игнорируя аргументы
        let (cmd, _) = match command.iter().position(|&byte| byte == b' ') {
            Some(pos) => command.split_at(pos),
            None => (command, &[][..]),
        };

        // Преобразуем команду в строку и обрезаем пробелы и \0
        let comm = core::str::from_utf8(cmd).unwrap_or("").trim();

        // Подсчитываем количество ненулевых байтов
        let mut len = 0;
        for i in comm.as_bytes() {
            if *i != 0 {
                len += 1;
            }
        }

        let comm_len_bytes = number_to_ascii_bytes(len);

        // Выводим длину строки на экран
        for (i, &byte) in comm_len_bytes.iter().enumerate() {
            if byte != 0 {
                write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            }
        }
        delay(3000000);

        // Фильтруем только непустые и ненулевые байты и сохраняем их в массиве фиксированного размера
        let mut comm_filtered: [u8; 256] = [0; 256];
        let mut index = 0;
        for &byte in comm.as_bytes().iter() {
            if byte != 0 && !byte.is_ascii_whitespace() {
                comm_filtered[index] = byte;
                index += 1;
            }
        }

        // Печатаем команду для отладки
        for (i, &byte) in comm_filtered.iter().take(len).enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
        }
        delay(3000000);

        let commands: [Command; 7] = [
            Command::new("hello", hello_action),
            Command::new("time", time_action),
            Command::new("time_add", time_add_action),
            Command::new("error", error_action),
            Command::new("reboot", reboot_action),
            Command::new("shutdown", shutdown_action),
            Command::new("clear", clear),
        ];

        for cmd in commands.iter() {
            let mut cmd_name_bytes = [0u8; 256];
            let cmd_name_len = cmd.name.bytes().count();
            for (i, byte) in cmd.name.bytes().enumerate() {
                cmd_name_bytes[i] = byte;
            }

            // Печатаем имя команды для отладки
            for (i, &byte) in cmd_name_bytes.iter().take(cmd_name_len).enumerate() {
                write_char(row + 2, i, byte, 0x07); // Печатает на строке row + 2
            }
            delay(3000000);

            if comm_filtered[..len] == cmd_name_bytes[..cmd_name_len] {
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
