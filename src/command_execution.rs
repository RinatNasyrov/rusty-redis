use crate::RESPMessenge;
use std::collections::HashMap;
use std::collections::hash_map::Entry::Occupied;
use std::collections::hash_map::Entry::Vacant;
use std::thread;
use std::time::Duration;

pub fn execute_command(
    input: RESPMessenge,
    key_value_pairs: &mut HashMap<String, String>,
) -> String {
    let plain_command = input.get_plain_command();
    // Матчим именно срез потому, что со срезами матчинг работает нормально,
    // а сложные объекты с литералами сопоставлять не может, тк в расте не принято
    // чтобы встроенные операторы языка делали неявные преобразования
    match plain_command[..] {
        ["PING"] => "+PONG\r\n".to_string(),
        ["PING30"] => {
            // Долгий запрос для теста праллельной/ассинхронной обработки
            thread::sleep(Duration::from_secs(30));
            "+PONG30\r\n".to_string()
        }
        // Работа со строками
        ["SET", key_name, value] => {
            key_value_pairs.insert(key_name.to_string(), value.to_string());
            "+OK\r\n".to_string()
        }
        ["GET", key_name] => match key_value_pairs.entry(key_name.to_string()) {
            // .clone(..) подсказал линтер потому, что String не реализует трейт Copy
            // и видимо нельзя просто сделать move во внешнюю переменную при возврате
            // значения из функции
            Occupied(value) => format!("+{}\r\n", value.get().clone()),
            Vacant(_) => "$-1\r\n".to_string(),
        },
        // Работа с ключами
        ["EXISTS", key_name] => {
            let mut result = 0;
            if key_value_pairs.contains_key(key_name) {
                result = 1;
            }
            // Можно было бы проще в строку воткнуть result,
            // но иначе линтер давал ворнинг за неиспользую переменную
            format!(":{}\r\n", result).to_string()
        }
        ["COMMAND", "HELP"] => "Подробная справка\r\n".to_string(),
        _ => "-Неизвестная комманда\r\n".to_string(),
    }
}
