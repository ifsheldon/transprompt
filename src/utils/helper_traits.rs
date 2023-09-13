pub trait Ok {
    type OkType;
    fn ok(&self) -> bool;
}

impl<T> Ok for Option<T> {
    type OkType = T;
    fn ok(&self) -> bool {
        self.is_some()
    }
}

pub trait ThenDo: Ok {
    unsafe fn ok_val_unchecked(&self) -> &Self::OkType;
    fn ok_then_do(&self, then_do: impl FnOnce(&Self::OkType)) -> &Self {
        if self.ok() {
            then_do(unsafe { self.ok_val_unchecked() });
        }
        self
    }
    fn ok_then_do_otherwise(&self, then_do: impl FnOnce(&Self::OkType), otherwise_do: impl FnOnce(&Self)) {
        if self.ok() {
            then_do(unsafe { self.ok_val_unchecked() });
        } else {
            otherwise_do(self);
        }
    }
}

pub trait ThenDoMut: Ok {
    unsafe fn ok_val_unchecked(&mut self) -> &mut Self::OkType;
    fn ok_then_do_mut(&mut self, then_do: impl FnOnce(&mut Self::OkType)) -> &mut Self {
        if self.ok() {
            then_do(unsafe { self.ok_val_unchecked() });
        }
        self
    }
    fn ok_then_do_otherwise_mut(&mut self, then_do: impl FnOnce(&mut Self::OkType), otherwise_do: impl FnOnce(&mut Self)) {
        if self.ok() {
            then_do(unsafe { self.ok_val_unchecked() });
        } else {
            otherwise_do(self);
        }
    }
}

impl<T> ThenDo for Option<T> {
    unsafe fn ok_val_unchecked(&self) -> &Self::OkType {
        self.as_ref().unwrap_unchecked()
    }
}

impl<T> ThenDoMut for Option<T> {
    unsafe fn ok_val_unchecked(&mut self) -> &mut Self::OkType {
        self.as_mut().unwrap_unchecked()
    }
}