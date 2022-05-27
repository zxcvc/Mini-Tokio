async fn ff(){
    df().await;
    println!("fsfsdf");
}
async fn df(){
    println!("12121");
    thread::sleep(Duration::from_secs_f32(5.0));
}
fn main() {
    let tokio = MiniTokio::new();
    tokio.block_on(ff());
}

use futures::{task::ArcWake, FutureExt};
use std::{future::Future, thread, time::Duration};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Wake};

struct Task {
    sender: crossbeam::channel::Sender<Arc<Self>>,
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
}
impl Task {
    fn new(
        sender: crossbeam::channel::Sender<Arc<Self>>,
        future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    ) -> Self {
        Self { sender, future }
    }

    fn poll(self: Arc<Self>) {
        let wk = futures::task::waker(self.clone());
        let mut cx = Context::from_waker(&wk);
        let mut future = self.future.try_lock().unwrap();
        future.as_mut().poll(&mut cx);
    }
}
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.sender.send(arc_self.clone()).unwrap();
    }
}

struct MiniTokio {
    reciver: crossbeam::channel::Receiver<Arc<Task>>,
    sender: crossbeam::channel::Sender<Arc<Task>>,
}

impl MiniTokio {
    fn new() -> Self {
        let (sender, reciver) = crossbeam::channel::unbounded();
        Self { reciver, sender }
    }

    pub fn run(&self) -> thread::JoinHandle<()>{
        let rec = self.reciver.clone();
        thread::spawn(move ||{
            for item in rec.iter() {
                item.poll();
            }
        })
    }

    fn get_sender(&self) -> crossbeam::channel::Sender<Arc<Task>> {
        self.sender.clone()
    }

    fn block_on<T: Future<Output = ()> + Send + 'static>(self, future: T) {
        let j = self.run();
        let task = Task::new(self.get_sender(), Mutex::new(Box::pin(future)));
        let task = Arc::new(task);
        task.wake();
        drop(self);
        j.join();
    }
}
