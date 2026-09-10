use rusty_redis::*;
use std::{io::BufReader, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // Пока обрабатываем все в основном потоке, потом вынесем в отдельные
        let mut reader = BufReader::new(&stream);
        loop {
            process_input(read_input(&mut reader));
        }
    }
}
fn process_input(input: Input) {
    println!("{input:?}");
}
