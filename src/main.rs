use rusty_redis::*;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::{io::BufReader, net::Shutdown, net::TcpListener, net::TcpStream};

fn main() {
    // Соединение и многопоточность
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    // Структуры данных
    let kv: HashMap<String, String> = HashMap::new();
    let kv_mutex = Arc::new(Mutex::new(kv));

    // Полученные потоки ввода-вывода сохраним чтобы погасить, когда перестанем слушать
    // входящие соединения
    // Погасим потоки - закончатся циклы в обработчиках
    let mut stream_vec: Vec<TcpStream> = Vec::new();

    for connection in listener.incoming() {
        let stream = connection.unwrap();
        let stream_clone = stream.try_clone().unwrap();
        stream_vec.push(stream_clone);

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
        pool.execute(move || {
            handle_connection(stream, Arc::clone(&value));
        });
        // Иммитация прерывания работы
        break;
    }

    // Гасим потоки ввода-вывода, когда перестали слушать соединения
    for stream in stream_vec {
        let _ = stream.shutdown(Shutdown::Both);
    }
}

fn handle_connection(stream: TcpStream, mutex_arc: Arc<Mutex<HashMap<String, String>>>) {
    let mut reader = BufReader::new(&stream);
    loop {
        // Пока команда не прочитана полностью, обработка
        // не начнется, а значит структуры данных типа словаря
        // не будут все время заняты
        let messenge = read_resp_messenge(&mut reader);
        // Для отладки, потом вывести в лог
        println!("{messenge:?}");

        let mut mutex = mutex_arc.lock().unwrap();
        let response = execute_command(messenge, &mut mutex);
        // Здесь прикол чтобы обойти ограничение сосуществование
        // мутабельной и иммутабельной ссылок, вызываем метод на
        // иммутабельной ссылке, для которой реализован трейт Write
        // как для самого типа
        _ = (&stream).write_all(response.as_bytes());
    }
}
