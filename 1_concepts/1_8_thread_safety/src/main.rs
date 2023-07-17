use std::{
    cell::Cell,
    rc::Rc,
    sync::{Mutex, MutexGuard},
    thread,
};

#[derive(Debug)]
struct OnlySync<'a>(MutexGuard<'a, ()>);

#[derive(Debug)]
struct OnlySend(Cell<()>);

#[derive(Debug)]
struct SyncAndSend(Mutex<()>);

#[derive(Debug)]
struct NotSyncNotSend(Rc<()>);

fn main() {
    let both = SyncAndSend(Mutex::new(()));
    let sync = OnlySync(both.0.lock().unwrap());

    thread::scope(|s| {
        s.spawn(|| {
            println!("{:?}", &both);
            println!("{:?}", &sync);
        });
    });

    drop(sync);

    let send = OnlySend(Cell::new(()));
    thread::spawn(move || {
        println!("{:?}", both);
        println!("{:?}", send);
    })
    .join()
    .unwrap();
}
