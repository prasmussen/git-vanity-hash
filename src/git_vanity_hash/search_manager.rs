use std::sync::mpsc;


pub struct SearchManager<T> {
    workers: Vec<mpsc::Sender<()>>,
    found_tx: mpsc::Sender<T>,
    found_rx: mpsc::Receiver<T>,
}

impl<T> SearchManager<T> {
    pub fn new() -> SearchManager<T> {
        let (found_tx, found_rx) = mpsc::channel();

        SearchManager{
            workers: vec![],
            found_tx,
            found_rx,
        }
    }

    pub fn new_worker(&mut self) -> Worker<T> {
        let (cancel_sender, cancel_receiver) = mpsc::channel();
        self.workers.push(cancel_sender);

        Worker::new(cancel_receiver, self.found_tx.clone())
    }

    // This drops the found_tx channel which is important
    // because otherwise the receiver could block forever
    pub fn immutable(self) -> ImmutableSearchManager<T> {
        ImmutableSearchManager{
            workers: self.workers,
            found_rx: self.found_rx,
        }
    }
}


pub struct ImmutableSearchManager<T> {
    workers: Vec<mpsc::Sender<()>>,
    found_rx: mpsc::Receiver<T>,
}

impl<T> ImmutableSearchManager<T> {
    pub fn race(self) -> Option<T> {
        match self.found_rx.recv() {
            Ok(x) => {
                self.stop_all_workers();
                Some(x)
            },

            Err(_) =>
                None,
        }
    }

    fn stop_all_workers(&self) {
        for chan in &self.workers {
            let _ = chan.send(());
        }
    }
}


pub struct Worker<T> {
    stop_rx: mpsc::Receiver<()>,
    found_tx: mpsc::Sender<T>,
}

impl<T> Worker<T> {
    pub fn new(stop_rx: mpsc::Receiver<()>, found_tx: mpsc::Sender<T>) -> Worker<T> {
        Worker{
            stop_rx,
            found_tx,
        }
    }

    pub fn should_stop(&self) -> bool {
        match self.stop_rx.try_recv() {
            Ok(()) =>
                true,

            Err(_) =>
                false,
        }
    }

    pub fn found(&self, x: T) {
        let _ = self.found_tx.send(x);
    }
}
