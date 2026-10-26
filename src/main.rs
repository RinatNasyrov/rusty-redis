use rusty_redis::*;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::{io::BufReader, net::TcpListener, net::TcpStream};

fn main() {
    // Соединение и многопоточность
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    // Структуры данных
    let kv: HashMap<String, String> = HashMap::new();
    let kv_mutex = Arc::new(Mutex::new(kv));

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // Передаем объект в замыкание
        // мьютекс - очевидно нужен чтоб избежать состояния гонки
        // Arc + клонирование нужно чтоб передать само значение (move)
        // в замыкание, ссылку туда отдать нельзя отдать, тк нельзя гарантировать,
        // что она переживет (лайфтайм) замыкание, Arc нужен чтоб у значения было несколько владельцев
        // и таким образом kv_mutex не перестает быть валидной переменной после первой итерации,
        // когда замыкание забирает владение
        // ===================================
        // Вот что проискодит кароч, из Arc объекта делаем копии и суем в замыкание, отдавая владение
        let value = kv_mutex.clone();

        // Передаем неизменяемую ссылку, чтобы не отдавать в функцию владение
        // и переменная не становилась невалидной к следующей итерации цикла
        pool.execute(move || {
            handle_connection(stream, Arc::clone(&value));
        });
    }
}

fn handle_connection(stream: TcpStream, kv_mutex: Arc<Mutex<HashMap<String, String>>>) {
    let mut reader = BufReader::new(&stream);
    loop {
        // Пока команда не прочитана полностью, обработка
        // не начнется, а значит структуры данных типа словаря
        // не будут все время заняты
        let messenge = read_resp_messenge(&mut reader);
        // Для отладки, потом вывести в лог
        println!("{messenge:?}");

        let mut kv = kv_mutex.lock().unwrap();
        let response = execute_command(messenge, &mut kv);
        // Здесь прикол чтобы обойти ограничение сосуществование
        // мутабельной и иммутабельной ссылок, вызываем метод на
        // иммутабельной ссылке, для которой реализован трейт Write
        // как для самого типа
        _ = (&stream).write_all(response.as_bytes());
    }
}
