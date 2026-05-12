#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct SYSTEMTIME {
    pub wYear: u16,
    pub wMonth: u16,
    pub wDayOfWeek: u16,
    pub wDay: u16,
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
    pub wMilliseconds: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct PSYSTEMTIME(*mut SYSTEMTIME);

impl PSYSTEMTIME {
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

impl AsRef<SYSTEMTIME> for PSYSTEMTIME {
    fn as_ref(&self) -> &SYSTEMTIME {
        unsafe { &*self.0 }
    }
}

impl AsMut<SYSTEMTIME> for PSYSTEMTIME {
    fn as_mut(&mut self) -> &mut SYSTEMTIME {
        unsafe { &mut *self.0 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct LPSYSTEMTIME(*mut SYSTEMTIME);

impl LPSYSTEMTIME {
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

impl AsRef<SYSTEMTIME> for LPSYSTEMTIME {
    fn as_ref(&self) -> &SYSTEMTIME {
        unsafe { &*self.0 }
    }
}

impl AsMut<SYSTEMTIME> for LPSYSTEMTIME {
    fn as_mut(&mut self) -> &mut SYSTEMTIME {
        unsafe { &mut *self.0 }
    }
}
