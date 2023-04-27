use std::ops::{Deref, DerefMut};

pub struct SwapBuffer<T> {
    active: Vec<T>,
    inactive: Vec<T>,
}

impl<T> Default for SwapBuffer<T> {
    fn default() -> Self {
        Self {
            active: Default::default(),
            inactive: Default::default(),
        }
    }
}

impl<T> Deref for SwapBuffer<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.active
    }
}

impl<T> DerefMut for SwapBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.active
    }
}

impl<T> SwapBuffer<T> {
    pub fn clear(&mut self) {
        self.active.clear();
        self.inactive.clear();
    }

    pub fn try_swap<F, I, Err>(&mut self, fun: F) -> Result<(), Err>
    where
        F: FnMut(T) -> Result<I, Err>,
        I: IntoIterator<Item = T>,
    {
        self.inactive.reserve(self.active.len());
        for elem in self.active.drain(..).map(fun) {
            if let Ok(res) = elem {
                self.inactive.extend(res)
            } else if let Err(e) = elem {
                return Err(e);
            }
        }

        std::mem::swap(&mut self.active, &mut self.inactive);

        Ok(())
    }
}
