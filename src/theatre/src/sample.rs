use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::pin::Pin;
use std::collections::HashMap;

// Определим тип сообщения для актора
enum ActorMessage {
    Command1(String),
    Command2(i32),
}

// Актор
struct MyActor;

impl MyActor {
    // Command dispatcher
    async fn handle_command(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::Command1(data) => {
                println!("Handling Command1 with data: {}", data);
                // Логика обработки Command1
            }
            ActorMessage::Command2(value) => {
                println!("Handling Command2 with value: {}", value);
                // Логика обработки Command2
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut channels: HashMap<String, UnboundedReceiver<ActorMessage>> = HashMap::new();

    // Инициализируем актора
    let mut actor = MyActor;

    // Создаем канал и добавляем его в HashMap
    let (tx1, rx1): (UnboundedSender<ActorMessage>, UnboundedReceiver<ActorMessage>) = unbounded_channel();
    channels.insert("channel_1".to_string(), rx1);

    // Запускаем задачу для обработки каналов
    tokio::spawn(async move {
        let mut futures: FuturesUnordered<Pin<Box<dyn futures::Future<Output = (String, Option<ActorMessage>)> + Send>>> = FuturesUnordered::new();

        // Функция для добавления каналов в futures
        fn add_channel_to_futures(
            futures: &mut FuturesUnordered<Pin<Box<dyn futures::Future<Output = (String, Option<ActorMessage>)> + Send>>>,
            channel_name: String,
            mut receiver: UnboundedReceiver<ActorMessage>,
        ) {
            futures.push(Box::pin(async move {
                let msg = receiver.recv().await;
                (channel_name, msg)
            }));
        }

        // Добавляем первый канал в список задач
        for (channel_name, receiver) in channels.drain() {
            add_channel_to_futures(&mut futures, channel_name, receiver);
        }

        loop {
            tokio::select! {
                Some((channel_name, Some(msg))) = futures.next() => {
                    println!("Received message from {}", channel_name);
                    actor.handle_command(msg).await;

                    // После обработки добавляем канал обратно в набор futures
                    if let Some(receiver) = channels.remove(&channel_name) {
                        add_channel_to_futures(&mut futures, channel_name, receiver);
                    }
                },
                else => {
                    println!("No more messages or channels");
                    break;
                }
            }
        }
    });

    // Отправка сообщений в канал
    tx1.send(ActorMessage::Command1("Hello from dynamic channel".to_string())).unwrap();

    // Подождем, чтобы актор обработал сообщения
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}