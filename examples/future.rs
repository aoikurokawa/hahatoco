use std::{
    future::Future,
    net::TcpStream,
    task::Poll,
    time::{Duration, Instant}, pin::Pin,
};

#[tokio::main]
async fn main() {
    /*
    let x = TcpStream::connect("127.0.0.1").unwrap();
    let y = TcpStream::connect("127.0.0.1").unwrap();

    x.write("foobar");
    y.write("foobar");

    assert_eq!(x.read(), "barfoo");
    assert_eq!(y.read(), "barfoo");
    */

    /*
    future
    let fut_x = TcpStream::connect("127.0.0.1")
        .and_then(|c| c.write("foobar"))
        .and_then(|c| c.read())
        .and_then(|(c, b)| b == "barfoo");
    println!("{:?}", fut);

    let fut_y = TcpStream::connect("127.0.0.1")
        .and_then(|c| c.write("foobar"))
        .and_then(|c| c.read())
        .and_then(|(c, b)| b == "barfoo");
    println!("{:?}", fut);

    a.spawn(fut_x);

    let server = TcpListener::new("127.0.0.1:1234")
        .incoming()
        .for_each(|s: TcpStream| {
            // tokio::spawn(fut) => tokio::executor::Handle::current().spawn(fut);
            // tokio::spawn(fut) => tokio::runtime::Handle::current().spawn(fut);

            tokio::spawn(ClientConnection::new(s))
        });

    */

    /*
    let a: Executor;
    let x = a.run(fut_x);
    let y = a.run(fut_y);


    let xy = a.run_all(vec![fut_x, fut_y]);
    */

    let when = Instant::now() + Duration::from_millis(10);
    let future = Delay { when };

    let out = future.await;
    assert_eq!(out, "done");
}

/*
struct Executor<F: Future>(Arc<Mutex<Vec<bool>>>, Vec<F>;

impl Executor {
    fn spawn<F>(&mut self, fut: F) where F: Future {
        self.1.push(fut);
    }

    fn run_all<F>(&mut self, futures: Vec<F>) -> Vec<(usize, Result<F::Item, F::Error>)> where F: Future {
        let mut done = 0;
        let mut results = Vec::with_capacity(futures.len());
        let mut tasks = Vec::new();
        let waiting_for: HashMap<(FD, Operation), Task>;

        for _  in 0..futures.len() {
            tasks.push(Task::new());
        }

        while done != futures.len() {
            for (i, f) in futures.iter_mut().enumerate() {
                // don't poll futures that can't make progress
                if !tasks[i].notified() {
                    continue;
                }

                task::set_current(tasks[i].clone());
                match f.poll() {
                    Ok(Async::Ready(t)) => {
                        // done
                        results.push((i, Ok(t)));
                    }
                    Err(e) => {
                        results.push((i, Err(e)));
                        done += 1
                    }
                    Ok(Async::NotReady) => {
                        // f *must* have arranged for tasks[i] (its task) to be notified later
                        continue;
                    }
                }
            }

            // wait for Task::notify to be called
            let select = waiting_for.keys().collect();
            for (fd, op) in epoll(select) {
                waiting_for.remove((fd, op)).notify();
            }
        }

        results
    }
}
*/

/*
struct PrintBytesRead {
    // But with O_NONBLOCKING set
    fd: std::net::TcpStream,
}
*/

/*
impl Future for PrintBytesRead {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        loop {
            match self.fd.read() {
                Ok(r) => {
                    eprintln!("got {} bytes", r.len());
                }
                Err(io::Error::WouldBlock) => {
                    // do something to make sure we are waken up
                    let reacotor = Handle::current();
                    match PollEvented::new_with_handle(self.fd, reactor).poll_read_ready() {
                        Ok(Async::Ready(_)) => {
                            // socket became ready between when we read an called
                            // poll_ready_ready()
                            continue;
                        },
                        Ok(Async::NotReady() => return Ok(Async::NotReady),
                        Err(e_ => return Err(e),
                    }
                    return Ok(Async::NotReady);
                }
                Err(io::Error::Closed) => {
                    return Ok(Async::Ready);
                }
                Err(e) => {
                    return Err(e);
                }

            }
        }
    }
}
*/

/*
enum Operaiton {
    Read,
    Write,
}
*/

/*
fn reactor_thread(notify_me: mpsc::Receiver<(Task, FD, Operation)> {

    loop {
        // accept new things to watch for
        while let Some((task, fd, op)) = notify_me.try_recv() {
            waiting_for.insert((fd, op), task);
        }

    }
}
*/

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

enum MainFuture {
    State0,
    State1(Delay),
    Terminated,
}

impl Future for MainFuture {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        loop {
            match *self {
                MainFuture::State0 => {
                    let when = Instant::now() + Duration::from_millis(10);
                    let future = Delay { when };
                    *self = MainFuture::State1(future);
                }
                MainFuture::State1(ref mut my_future) => {
                    match Pin::new(my_future).poll(cx) {
                        Poll::Ready(out) => {
                            assert_eq!(out, "done");
                            *self = MainFuture::Terminated;
                            return Poll::Ready(());
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }

                }
                MainFuture::Terminated => {
                    panic!("future polled after completion")
                }
            }
        }
    }
}
