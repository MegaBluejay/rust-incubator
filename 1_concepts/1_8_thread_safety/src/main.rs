use std::{
    cell::Cell,
    marker::PhantomData,
    rc::Rc,
    sync::{Mutex, MutexGuard},
    thread,
};

#[derive(Debug)]
struct OnlySync<'a>(PhantomData<MutexGuard<'a, ()>>);

#[derive(Debug)]
struct OnlySend(PhantomData<Cell<()>>);

#[derive(Debug)]
struct SyncAndSend(PhantomData<Mutex<()>>);

#[derive(Debug)]
struct NotSyncNotSend(PhantomData<Rc<()>>);

fn main() {
    let sync = OnlySync(PhantomData);
    let send = OnlySend(PhantomData);
    let both = SyncAndSend(PhantomData);
    let none = NotSyncNotSend(PhantomData);

    thread::scope(|s| {
        s.spawn(|| {
            println!("{:?}", &sync);
        });

        s.spawn(|| {
            println!("{:?}", &both);
        });

        // s.spawn(|| {
        //     println!("{:?}", &send);
        // });

        // s.spawn(|| {
        //     println!("{:?}", &none);
        // })
    });

    thread::scope(|s| {
        s.spawn(move || {
            println!("{:?}", send);
        });

        s.spawn(move || {
            println!("{:?}", both);
        });

        // s.spawn(move || {
        //     println!("{:?}", sync);
        // });

        // s.spawn(move || {
        //     println!("{:?}", none);
        // });
    });

    println!("{:?}", none);
}
