use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

// Thread pool that accepts jobs and threads handle them when they
// become available
pub struct ThreadPoolQ<T> {
    queue: Arc<Mutex<Option<VecDeque<T>>>>,
    threads: Vec<Option<thread::JoinHandle<()>>>,
    cvar: Arc<(Condvar, Mutex<bool>)>,
}

impl<T> ThreadPoolQ<T>
where
    T: Send + 'static,
{
    pub fn new(size: usize, f: impl Fn(T) -> () + Send + Sync + 'static) -> ThreadPoolQ<T> {
        let mut threads: Vec<Option<thread::JoinHandle<()>>> = Vec::with_capacity(size);
        let q: Arc<Mutex<Option<VecDeque<T>>>> = Arc::new(Mutex::new(Some(VecDeque::new())));
        let cvar = Arc::new((Condvar::new(), Mutex::new(true)));
        let f = Arc::new(f);
        for _ in 0..size {
            let q = Arc::clone(&q);
            let cvar = Arc::clone(&cvar);
            let f = Arc::clone(&f);
            threads.push(Some(thread::spawn(move || loop {
                let mut queue = q.lock().unwrap();
                if queue.is_none() {
                    return;
                }
                let job = queue.as_mut().unwrap().pop_front();
                drop(queue);
                if job.is_none() {
                    let mut lock = cvar.1.lock().unwrap();
                    *lock = false;
                    drop(lock);
                    drop(cvar.0.wait(cvar.1.lock().unwrap()));
                    continue;
                }
                if job.is_some() {
                    f(job.unwrap());
                }
            })));
        }
        ThreadPoolQ {
            queue: q,
            threads,
            cvar,
        }
    }

    pub fn push_job(&mut self, job: T) {
        let mut q = self.queue.lock().unwrap();
        q.as_mut().unwrap().push_back(job);
        let mut lock = self.cvar.1.lock().unwrap();
        *lock = true;
        self.cvar.0.notify_all();
    }
}

impl<T> Drop for ThreadPoolQ<T> {
    fn drop(&mut self) {
        loop {
            let q = self.queue.lock().unwrap();
            if q.is_some() && q.as_ref().unwrap().is_empty() {
                break;
            }
            drop(q);
        }
        let mut q = self.queue.lock().unwrap();
        q.take();
        drop(q);
        let mut lock = self.cvar.1.lock().unwrap();
        *lock = true;
        drop(lock);
        self.cvar.0.notify_all();
        for t in &mut self.threads {
            if let Some(t) = t.take() {
                t.join().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn creates_a_new_threadpool() {
        let tp = ThreadPoolQ::<usize>::new(5, |_| {
            thread::sleep(Duration::from_secs(1));
        });
        assert_eq!(5, tp.threads.capacity());
        let q = tp.queue.lock().unwrap();
        assert!(q.is_some());
    }

    #[test]
    fn handles_jobs_in_order() {
        let mut tp = ThreadPoolQ::new(1, |num: usize| {
            println!("Received: {}", num);
            thread::sleep(Duration::from_secs(1));
        });
        tp.push_job(1);
        tp.push_job(2);
        thread::sleep(Duration::from_secs(1));
        let q = tp.queue.lock().unwrap();
        assert_eq!(q.as_ref().unwrap().len(), 1);
        let mut nq: VecDeque<usize> = VecDeque::new();
        nq.push_back(2);
        assert_eq!(q.as_ref().unwrap(), &nq);
    }
}
