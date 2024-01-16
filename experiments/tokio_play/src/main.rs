#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

async fn local() {
    let nonsend_data = Rc::new("world");
    let local = tokio::task::LocalSet::new();

    let nonsend_data2 = nonsend_data.clone();
    let util1 = local.run_until(async move {
        // ...
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        println!("hello {}", nonsend_data2)
    });

    let util2 = local.run_until(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("goodbye {}", nonsend_data)
    });



    util1.await;
    util2.await;
}

// async fn main1() {
//     #[derive(Debug)]
//     enum Message1 {
//         Ping,
//         Pong
//     }
//
//     #[derive(Debug)]
//     enum ReqMsg {
//         GET
//     }
//
//     #[derive(Debug)]
//     enum RespMsg {
//         Ok,
//         Err,
//     }
//
//     async fn process1(mut rx1: Receiver<Message1>, mut req_rx: Receiver<ReqMsg>, resp_tx: Sender<RespMsg>){
//
//     }
//
//     let (tx1, mut rx1) = tokio::sync::mpsc::channel::<Message1>(10);
//     let (req_tx, mut req_rx) = tokio::sync::mpsc::channel::<ReqMsg>(10);
//     let (resp_tx, mut resp_rx) = tokio::sync::mpsc::channel::<RespMsg>(10);
//     tokio::spawn(async  {
//         println!("started 1 process");
//         let local = tokio::task::LocalSet::new();
//
//         local.run_until(async {
//             local.spawn_local(async  {
//                 println!("started 1.1 process");
//                 loop {
//                     println!("1.1 loop");
//                     // if let Some(msg) = rx1.recv().await {
//                     //     println!("msg1: {msg:?}")
//                     // }
//                 }
//             });
//             // local.spawn_local(async  {
//             //     println!("started 1.2 process");
//             //     loop {
//             //         println!("1.2 loop");
//             //         // if let Some(msg) = req_rx.recv().await {
//             //         //     println!("req: {msg:?}");
//             //         //     resp_tx.send(RespMsg::Ok).await;
//             //         // }
//             //     }
//             // });
//             // handle.await;
//         }).await;
//
//         Ok::<(), std::io::Error>(())
//     });
//
//     // let process2_handle = tokio::spawn(async move {
//     //     let local = tokio::task::LocalSet::new();
//     //     local.spawn_local(async move {
//     //         loop {
//     //             if let Some(msg) = resp_rx.recv().await {
//     //                 println!("resp: {msg:?}")
//     //             }
//     //         }
//     //     });
//     // });
//
//
//     tx1.send(Message1::Ping);
//     req_tx.send(ReqMsg::GET);
//
//     // tokio::join!(process1_handle, process2_handle);
//     tokio::time::sleep(Duration::from_secs(10)).await;
// }

trait Message {}


#[tokio::main]
async fn main() {
    // use futures::FutureExt;
    // use futures::future::select_all;
    // use tokio::sync::mpsc;
    //
    // // Предположим, что Message и Receiver уже определены
    // let receivers: Vec<mpsc::Receiver<String>> = vec![/* ... */];
    //
    // let mut futures = Vec::new();
    // for mut receiver in receivers {
    //     let future = Box::pin(async move {
    //         let option = receiver.recv().await;
    //         option
    //     });
    //     futures.push(future);
    // }
    //
    // while !futures.is_empty() {
    //     let (result, index, remaining) = select_all(futures).await;
    //     futures = remaining;
    //
    //     if let Some(msg) = result {
    //         println!("msg: {msg}");
    //     }
    // }

    let (tx, rx1) = watch::channel::<i32>(100500);
    // WatchStream::new()
    let mut stream1 = tokio_stream::wrappers::WatchStream::new(rx1.clone());
    let mut stream2 = tokio_stream::wrappers::WatchStream::new(rx1);
    let value = stream1.next().await.unwrap();
    println!("value 1: {value:?}");
    let value = stream2.next().await.unwrap();
    println!("value 2: {value:?}");


    let (_, rx2) = watch::channel::<i32>(100500);
    // WatchStream::new()
    let mut stream1 = tokio_stream::wrappers::WatchStream::new(rx2);

    // WatchStream::from(&)
}