use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    // В документации еще предлагают попробовать заменить new на build с такой сигной:
    // pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError>
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // Здесь ровно тот же прикол, что и при передаче мьютекса
            // со словарем в обработчик в main'е
            // (оказывается можно было не разбираться самому, а перечитать док)
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        // Берем содержимое option'а
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        // Просто притерировать воркеров нельзя,
        // тк join() забирает владение и компилятор ругается на это
        // drain() же извлекает каждый элемент вектора с концами прям
        // чтоб его можно было отдать
        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);

            // По идее основной поток будет ждать завершения рабочих потоков
            // при завершении работы программы
            worker.thread.join().unwrap();
        }
    }
}

// Структура нужная чтобы реализовать поведение
// выполнения/ожидания передаваемых операций
// Нужна для реализации ThreadPool поэтому не публичная
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // Примечание из документации
                // while let Ok(job) = receiver.lock().unwrap().recv()
                // так нельзя делать здесь, тк while let/if let/match
                // не отпустит мьютекс канала пока весь блок вместе с job
                // не выполнится полностью, а значит другие потоки будут простаивать
                let message = receiver.lock().unwrap().recv();

                // При закрытии sender'а перестаем читать его в цикле и позволяем
                // воркеру завершиться
                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");

                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}
