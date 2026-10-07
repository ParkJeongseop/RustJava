use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Gives the executor one turn: pending on the first poll (after waking its own task), ready on
/// the second. The interpreter awaits it when a thread has used up its time slice.
#[derive(Default)]
pub struct TimeSliceEnd {
    yielded: bool,
}

impl Future for TimeSliceEnd {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            return Poll::Ready(());
        }

        self.yielded = true;
        context.waker().wake_by_ref();

        Poll::Pending
    }
}
