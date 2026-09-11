use crate::RESPMessenge::*;
use std::io::Read;
use std::{
    io::{BufReader, prelude::*},
    net::TcpStream,
};

#[derive(Debug)]
pub enum RESPMessenge {
    SimpleString(String),
    ErrorMessenge(String),
    Integer(usize),
    BulkString(String),
    Array(Vec<RESPMessenge>),
    None,
}

impl RESPMessenge {
    pub fn get_plain_command(&self) -> Vec<&str> {
        let mut result = Vec::new();
        self.collect_command_parts(&mut result);
        result
    }

    fn collect_command_parts<'a>(&'a self, buffer: &mut Vec<&'a str>) {
        match self {
            SimpleString(value) | BulkString(value) => buffer.push(value.as_str()),
            Array(values) => values.iter().for_each(|v| v.collect_command_parts(buffer)),
            _ => return,
        }
    }
}

pub fn read_resp_messenge(mut reader: &mut BufReader<&TcpStream>) -> RESPMessenge {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    match line.chars().next().unwrap() {
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
            let size: usize = line[1..].trim().parse().expect("Не целое число");
            BulkString(read_bulk_string(size, &mut reader))
        }
        '*' => {
            let size: usize = line[1..].trim().parse().expect("Не целое число");
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

fn read_array(size: usize, reader: &mut BufReader<&TcpStream>) -> Vec<RESPMessenge> {
    let mut counter = size;
    let mut result: Vec<RESPMessenge> = Vec::new();
    while counter > 0 {
        result.push(read_resp_messenge(reader));
        counter -= 1;
    }

    result
}
