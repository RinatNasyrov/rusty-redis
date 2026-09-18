use rusty_redis::*;
use std::collections::HashMap;
use std::io::Write;
use std::{io::BufReader, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let mut key_value_pairs: HashMap<String, String> = HashMap::new();

        // Пока обрабатываем все в основном потоке, потом вынесем в отдельные
        let mut reader = BufReader::new(&stream);
        loop {
            let messenge = read_resp_messenge(&mut reader);
            // Для отладки, потом вывести в лог
            println!("{messenge:?}");

            let response = execute_command(messenge, &mut key_value_pairs);
            // Здесь прикол чтобы обойти ограничение сосуществование
            // мутабельной и иммутабельной ссылок, вызываем метод на
            // иммутабельной ссылке, для которой реализован трейт Write
            // как для самого типа
            _ = (&stream).write_all(response.as_bytes());
        }
    }
}
