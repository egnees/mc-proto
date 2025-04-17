use std::any::Any;

use super::oneshot::{channel, Receiver, RecvError, Sender};

////////////////////////////////////////////////////////////////////////////////

pub struct Trigger(Sender<Box<dyn Any>>);

impl Trigger {
    pub fn invoke<T: Any>(self, value: T) -> Result<(), T> {
        let result = self.0.send(Box::new(value));
        if let Err(x) = result {
            Err(*x.downcast::<T>().unwrap())
        } else {
            Ok(())
        }
    }

    pub fn has_waiter(&self) -> bool {
        self.0.has_receiver()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Waiter(Receiver<Box<dyn Any>>);

impl Waiter {
    pub async fn wait<T: Any>(self) -> Result<T, RecvError> {
        let result = self.0.await?;
        let result = result.downcast::<T>().expect("dynamic cast error");
        Ok(*result)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn make_trigger() -> (Waiter, Trigger) {
    let (sender, receiver) = channel();
    (Waiter(receiver), Trigger(sender))
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use tokio::task::LocalSet;

    use crate::util::trigger::make_trigger;

    ////////////////////////////////////////////////////////////////////////////////

    #[tokio::test]
    async fn basic() {
        let rt = LocalSet::new();
        let (waiter, trigger) = make_trigger();
        rt.spawn_local(async move {
            let x = waiter.wait::<i32>().await.unwrap();
            assert_eq!(x, 5);
        });
        rt.spawn_local(async move {
            trigger.invoke(5).unwrap();
        });
        rt.await;
    }
}
