use std::future::Future;
use std::task::{Context, Poll, Waker};
use std::pin::Pin;
use std::sync::Arc;
use crossbeam::atomic::AtomicCell;
use anyhow::*;


struct Status<T>{
    result:AtomicCell<Option<Result<T>>>,
    wake:AtomicCell<Option<Waker>>
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
        if let Some(wake)= self.right_status.wake.take(){
            wake.wake()
        }
    }
}

impl<L,R> Drop for Left<L,R>{
    #[inline]
    fn drop(&mut self) {
        self.my_status.result.store(Some(Err(anyhow!("left is drop"))));
        self.right_status.result.store(Some(Err(anyhow!("left is drop"))));
        if let Some(wake)= self.right_status.wake.take(){
            wake.wake()
        }
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
        if let Some(wake)= self.left_status.wake.take(){
            wake.wake()
        }
    }
}

impl<L,R> Drop for Right<L,R>{
    #[inline]
    fn drop(&mut self) {
        self.my_status.result.store(Some(Err(anyhow!("right is drop"))));
        self.left_status.result.store(Some(Err(anyhow!("right is drop"))));
        if let Some(wake)= self.left_status.wake.take(){
            wake.wake()
        }
    }
}



impl<T> Future for PipeHandler<T>{
    type Output = Result<T>;
    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

        self.my_status.wake.store(Some(cx.waker().clone()));
        if  let Some(r)=self.my_status.result.take(){
            Poll::Ready(r)
        }else{
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

