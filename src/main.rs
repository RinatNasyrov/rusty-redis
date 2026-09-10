// use std::io;
use std::io::BufRead;
use std::io::Read;
use std::{
    io::{self, BufReader, BufWriter, prelude::*},
    net::{TcpListener, TcpStream},
};

use crate::Input::*;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let mut reader = BufReader::new(&stream);
        let input = read_input(&mut reader);
        println!("{input:?}");
    }
}

#[derive(Debug)]
enum Input {
    SimpleString(String),
    ErrorMessenge(String),
    Integer(usize),
    BulkString(String),
    Array(Vec<Input>),
    None,
}

fn read_input(mut reader: &mut BufReader<&TcpStream>) -> Input {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    match line.chars().next().unwrap() {
        // Однострочники повторно читаем поучается!
        '+' => {
            let simple_string = String::from(line[1..].trim());
            SimpleString(simple_string)
        }
        '-' => {
            let error_messenge = String::from(line[1..].trim());
            ErrorMessenge(error_messenge)
        }
        ':' => {
            let number: usize = line[1..].trim().parse().expect("Не целое число");
            Integer(number)
        }
        '$' => {
            // TODO: Проверить, что такой способ извлечения размера точно рабочий
            let size: usize = line[1..].trim().parse().expect("bullshit input");
            BulkString(read_bulk_string(size, &mut reader))
        }
        '*' => {
            // TODO: Проверить, что такой способ извлечения размера точно рабочий
            let size: usize = line[1..].trim().parse().expect("bullshit input");
            Array(read_array(size, &mut reader))
        }
        _ => None,
    }
}

fn read_bulk_string(size: usize, reader: &mut BufReader<&TcpStream>) -> String {
    let mut buffer = vec![0; size];
    reader.read_exact(&mut buffer).unwrap();
    // Двигаем указатель в потоке, чтобы дать понять,
    // что перенос строки мы тоже прочитали
    // (просто считать ровно на два байта больше - не работает,
    // символы переноса читаются следующим read_line'ом)
    reader.consume(2);
    String::from_utf8(buffer).unwrap_or_default()
}

fn read_array(size: usize, reader: &mut BufReader<&TcpStream>) -> Vec<Input> {
    let mut counter = size;
    let mut result: Vec<Input> = Vec::new();
    while counter > 0 {
        result.push(read_input(reader));
        counter -= 1;
    }

    result
}
