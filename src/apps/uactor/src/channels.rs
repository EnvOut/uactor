fn channels() {
    // many to one
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    tx.send(());

    // many -> many
    let (tx, rx) = tokio::sync::broadcast::channel(10);
    tx.send(());

    // one -> many (only last sent visible)
    let (tx, rx) = tokio::sync::watch::channel(());
    tx.send(());

    // one -> one (only for reply)
    let (tx, rx) = tokio::sync::oneshot::channel();
    tx.send(());
    println!("Hello, world!");


    // let arc = Arc::new(L1 { tx });
    // arc.send(());

}
