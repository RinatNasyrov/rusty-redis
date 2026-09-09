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
        '+' => SimpleString(read_simple_string(&mut reader)),
        '-' => ErrorMessenge(read_simple_string(&mut reader)),
        ':' => Integer(read_integer(&mut reader)),
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

fn read_simple_string(reader: &mut BufReader<&TcpStream>) -> String {
    let mut line = String::new();
    reader.read_line(&mut line);
    line
}

fn read_integer(reader: &mut BufReader<&TcpStream>) -> usize {
    let mut line = String::new();
    reader.read_line(&mut line);
    print!("hii {line}");
    line.trim().parse().expect("Не целое число")
}

fn read_error_messenge(reader: &mut BufReader<&TcpStream>) -> String {
    let mut line = String::new();
    reader.read_line(&mut line);
    line
}

fn read_bulk_string(size: usize, reader: &mut BufReader<&TcpStream>) -> String {
    // \r\n прочитать еще
    let mut buffer = vec![0; size + 2];
    reader.read_exact(&mut buffer);
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
