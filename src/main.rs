use rusty_redis::*;
use std::{io::BufReader, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // Пока обрабатываем все в основном потоке, потом вынесем в отдельные
        let mut reader = BufReader::new(&stream);
        loop {
            execute_command(read_resp_messenge(&mut reader));
        }
    }
}
fn execute_command(input: RESPMessenge) {
    let plain_command = input.get_plain_command();
    // Матчим именно срез потому, что со срезами матчинг работает нормально,
    // а сложные объекты с литералами сопоставлять не может, тк в расте не принято
    // чтобы встроенные операторы языка делали неявные преобразования
    match plain_command[..] {
        ["PING"] => println!("PONG"),
        ["COMMAND", "HELP"] => println!("Подробная справка"),
        _ => println!("Неизвестная комманда"),
    }
}
