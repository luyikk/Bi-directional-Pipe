use crate::PipeError;
use std::cell::UnsafeCell;
use std::future::Future;
use std::io::{Error, ErrorKind, Result};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

struct Status<T> {
    result: UnsafeCell<Option<Result<T>>>,
    wake: UnsafeCell<Option<Waker>>,
}

pub struct PipeHandler<T> {
    my_status: Rc<Status<T>>,
}

pub struct Left<L, R> {
    my_status: Rc<Status<L>>,
    right_status: Rc<Status<R>>,
}

impl<L, R> Left<L, R> {
    #[inline]
    pub fn recv(&self) -> PipeHandler<L> {
        PipeHandler {
            my_status: self.my_status.clone(),
        }
    }
    #[inline]
    pub fn send(&self, v: R) {
        unsafe {
            *self.right_status.result.get() = Some(Ok(v));
            if let Some(wake) = (*self.right_status.wake.get()).take() {
                wake.wake()
            }
        }
    }
}

impl<L, R> Drop for Left<L, R> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            *self.my_status.result.get() =
                Some(Err(Error::new(ErrorKind::Other, PipeError::LeftDrop)));
            *self.right_status.result.get() =
                Some(Err(Error::new(ErrorKind::Other, PipeError::LeftDrop)));
            if let Some(wake) = (*self.right_status.wake.get()).take() {
                wake.wake()
            }
        }
    }
}

pub struct Right<L, R> {
    my_status: Rc<Status<R>>,
    left_status: Rc<Status<L>>,
}

impl<L, R> Right<L, R> {
    #[inline]
    pub fn recv(&self) -> PipeHandler<R> {
        PipeHandler {
            my_status: self.my_status.clone(),
        }
    }
    #[inline]
    pub fn send(&self, v: L) {
        unsafe {
            *self.left_status.result.get() = Some(Ok(v));
            if let Some(wake) = (*self.left_status.wake.get()).take() {
                wake.wake()
            }
        }
    }
}

impl<L, R> Drop for Right<L, R> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            *self.my_status.result.get() =
                Some(Err(Error::new(ErrorKind::Other, PipeError::RightDrop)));
            *self.left_status.result.get() =
                Some(Err(Error::new(ErrorKind::Other, PipeError::RightDrop)));
            if let Some(wake) = (*self.left_status.wake.get()).take() {
                wake.wake()
            }
        }
    }
}

impl<T> Future for PipeHandler<T> {
    type Output = Result<T>;
    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let this = Pin::into_inner(self);
            *this.my_status.wake.get() = Some(cx.waker().clone());
            if let Some(r) = (*this.my_status.result.get()).take() {
                Poll::Ready(r)
            } else {
                Poll::Pending
            }
        }
    }
}

#[inline]
pub fn pipe<L, R>() -> (Left<L, R>, Right<L, R>) {
    let left_status = Rc::new(Status {
        result: Default::default(),
        wake: Default::default(),
    });

    let right_status = Rc::new(Status {
        result: Default::default(),
        wake: Default::default(),
    });

    let left = Left {
        my_status: left_status.clone(),
        right_status: right_status.clone(),
    };

    let right = Right {
        my_status: right_status,
        left_status,
    };

    (left, right)
}
