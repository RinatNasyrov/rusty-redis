use rusty_redis::*;
use std::io::Write;
use std::thread;
use std::{io::BufReader, net::TcpListener, time::Duration};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // Пока обрабатываем все в основном потоке, потом вынесем в отдельные
        let mut reader = BufReader::new(&stream);
        loop {
            let response = execute_command(read_resp_messenge(&mut reader));
            // Здесь прикол чтобы обойти ограничение сосуществование
            // мутабельной и иммутабельной ссылок, вызываем метод на
            // иммутабельной ссылке, для которой реализован трейт Write
            // как для самого типа
            let write_result = (&stream).write_all(response.as_bytes());
            println!("{write_result:?}")
        }
    }
}
fn execute_command(input: RESPMessenge) -> String {
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
        ["COMMAND", "HELP"] => "Подробная справка\r\n".to_string(),
        _ => "-Неизвестная комманда\r\n".to_string(),
    }
}
