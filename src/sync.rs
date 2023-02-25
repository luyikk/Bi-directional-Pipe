use std::future::Future;
use std::task::{Context, Poll};
use std::pin::Pin;
use std::sync::Arc;
use crossbeam::atomic::AtomicCell;
use std::io::{Error, ErrorKind, Result};
use atomic_waker::AtomicWaker;
use crate::PipeError;


struct Status<T>{
    result:AtomicCell<Option<Result<T>>>,
    wake:AtomicWaker
}

pub struct PipeHandler<T>{
    my_status:Arc<Status<T>>
}

pub struct Left<L,R>{
    my_status:Arc<Status<L>>,
    right_status:Arc<Status<R>>
}

impl<L,R> Left<L,R>{
    #[inline]
    pub fn recv(&self)-> PipeHandler<L>{
        PipeHandler {
            my_status: self.my_status.clone()
        }
    }
    #[inline]
    pub fn send(&self,v:R){
        self.right_status.result.store(Some(Ok(v)));
        self.right_status.wake.wake();
    }
}

impl<L,R> Drop for Left<L,R>{
    #[inline]
    fn drop(&mut self) {
        self.my_status.result.store(Some(Err(Error::new(ErrorKind::Other,PipeError::LeftDrop))));
        self.right_status.result.store(Some(Err(Error::new(ErrorKind::Other,PipeError::LeftDrop))));
        self.right_status.wake.wake();
    }
}

pub struct Right<L,R>{
    my_status:Arc<Status<R>>,
    left_status:Arc<Status<L>>
}

impl<L,R> Right<L,R>{
    #[inline]
    pub fn recv(&self)-> PipeHandler<R>{
        PipeHandler {
            my_status: self.my_status.clone(),
        }
    }
    #[inline]
    pub fn send(&self,v:L){
        self.left_status.result.store(Some(Ok(v)));
        self.left_status.wake.wake();
    }
}

impl<L,R> Drop for Right<L,R>{
    #[inline]
    fn drop(&mut self) {
        self.my_status.result.store(Some(Err(Error::new(ErrorKind::Other,PipeError::RightDrop))));
        self.left_status.result.store(Some(Err(Error::new(ErrorKind::Other,PipeError::RightDrop))));
        self.left_status.wake.wake();
    }
}

impl<T> Future for PipeHandler<T>{
    type Output = Result<T>;
    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = Pin::into_inner(self);
        this.my_status.wake.register(cx.waker());
        if let Some(r) = this.my_status.result.take() {
            Poll::Ready(r)
        } else {
            Poll::Pending
        }
    }
}

#[inline]
pub fn pipe<L,R>()->(Left<L,R>,Right<L,R>){

    let left_status=Arc::new(Status{
        result: Default::default(),
        wake:  Default::default(),
    });

    let right_status=Arc::new(Status{
        result: Default::default(),
        wake:  Default::default(),
    });

    let left=Left{
        my_status:left_status.clone(),
        right_status:right_status.clone()
    };

    let right=Right{
        my_status:right_status,
        left_status
    };

    (left,right)
}

