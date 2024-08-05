use std::{collections::HashMap, marker::PhantomData, ptr::NonNull, sync::RwLock};

use once_cell::sync::Lazy;

use crate::kind::RErrorKind;

pub struct RError {
    data: Option<Repr>,
}

impl Drop for RError {
    fn drop(&mut self) {
        if self.data.is_some() {
            panic!("{}", self);
        }
    }
}

impl RError {
    pub fn ignore(self) {
        let mut this = self;
        let _ = this.data.take();
    }

    pub fn new_simple(module: RModule, kind: RErrorKind) -> Self {
        Self {
            data: Some(Repr::new_simple(module, kind)),
        }
    }

    pub fn new_simple_msg(module: RModule, kind: RErrorKind, msg: &'static str) -> Self {
        Self {
            data: Some(Repr::new_simple_msg(module, kind, msg)),
        }
    }

    pub fn new_custom_msg(module: RModule, kind: RErrorKind, msg: String) -> Self {
        Self {
            data: Some(Repr::new_custom_msg(module, kind, msg)),
        }
    }

    pub fn to_err<T>(self) -> std::result::Result<T, Self> {
        Err(self)
    }
}

impl std::fmt::Display for RError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data.as_ref() {
            Some(data) => write!(f, "{}", data),
            None => unreachable!(),
        }
    }
}

impl std::fmt::Debug for RError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

unsafe impl Send for RError {}

pub struct Repr {
    data: NonNull<()>,
    _mark: PhantomData<Box<ErrorData>>,
}

unsafe impl Send for Repr {}

impl Drop for Repr {
    fn drop(&mut self) {
        let bits = self.data.as_ptr() as usize;
        if (bits & Self::TAG_MASK) == 0 {
            let _ = unsafe { Box::from_raw(bits as *mut ErrorData) };
        }
    }
}

impl Repr {
    const TAG_SIMPLE: usize = 0b11;
    const TAG_MASK: usize = 0b11;

    #[inline]
    fn error_type_id(&self) -> u32 {
        let bits = self.data.as_ptr() as usize;
        if (bits & Self::TAG_MASK) == 0 {
            unsafe { (&*self.data.as_ptr().cast::<ErrorData>()).error_type_id() }
        } else {
            debug_assert_eq!(bits & Self::TAG_MASK, Self::TAG_SIMPLE);
            bits as u32
        }
    }

    fn new_simple(module: RModule, kind: RErrorKind) -> Self {
        let bits =
            (((module.0 as u32) << 16) | (kind.get_encode_code() as u32)) | Self::TAG_SIMPLE as u32;
        Self {
            data: unsafe { NonNull::new_unchecked(bits as usize as *mut _) },
            _mark: PhantomData,
        }
    }

    fn new_simple_msg(module: RModule, kind: RErrorKind, msg: &'static str) -> Self {
        let err_type_id = ((module.0 as u32) << 16) | (kind.get_encode_code() as u32);
        let mut err_data = Box::new(ErrorData::SimpleMessage((err_type_id, msg)));
        let res = Self {
            data: unsafe { NonNull::new_unchecked(err_data.as_mut() as *mut _ as *mut _) },
            _mark: PhantomData,
        };
        std::mem::forget(err_data);
        res
    }

    fn new_custom_msg(module: RModule, kind: RErrorKind, msg: String) -> Self {
        let err_type_id = ((module.0 as u32) << 16) | (kind.get_encode_code() as u32);
        let mut err_data = Box::new(ErrorData::CustomMessage((err_type_id, msg)));
        let res = Self {
            data: unsafe { NonNull::new_unchecked(err_data.as_mut() as *mut _ as *mut _) },
            _mark: PhantomData,
        };
        std::mem::forget(err_data);
        res
    }
}

impl PartialEq for Repr {
    fn eq(&self, other: &Self) -> bool {
        self.error_type_id() == other.error_type_id()
    }

    fn ne(&self, other: &Self) -> bool {
        self.error_type_id() != other.error_type_id()
    }
}

impl Eq for Repr {}

impl std::fmt::Display for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits = self.data.as_ptr() as usize;
        if (bits & Self::TAG_MASK) == 0 {
            let err_data = unsafe { &*self.data.as_ptr().cast::<ErrorData>() };
            let err_type_id = err_data.error_type_id();
            let module_id = (err_type_id >> 16) as u16;
            let kind = RErrorKind::from_encode_code(err_type_id as u16);
            let msg = err_data.error_msg();
            write!(
                f,
                "{}. {}. FROM `{}`",
                kind,
                msg,
                get_module_name(module_id)
            )
        } else {
            debug_assert_eq!(bits & Self::TAG_MASK, Self::TAG_SIMPLE);
            let kind = RErrorKind::from_encode_code((bits & 0xfff0) as u16);
            let module_id = (bits >> 16) as u16;
            write!(f, "{}. Null. FROM `{}`", kind, get_module_name(module_id))
        }
    }
}

impl std::fmt::Debug for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[repr(align(4))]
enum ErrorData {
    SimpleMessage((u32, &'static str)),
    CustomMessage((u32, String)),
}

impl ErrorData {
    #[inline]
    fn error_type_id(&self) -> u32 {
        match self {
            Self::CustomMessage((id, _)) => *id,
            Self::SimpleMessage((id, _)) => *id,
        }
    }

    fn error_msg(&self) -> &str {
        match self {
            Self::CustomMessage((_, msg)) => (*msg).as_str(),
            Self::SimpleMessage((_, msg)) => *msg,
        }
    }
}

static MODULE_LISTS: Lazy<RwLock<HashMap<u16, &'static str>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

fn get_module_name(module_id: u16) -> &'static str {
    unsafe {
        MODULE_LISTS
            .read()
            .unwrap_unchecked()
            .get(&module_id)
            .unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RModule(u16);

impl RModule {
    pub fn new(module_name: &'static str) -> Self {
        let mut hash_value = const_fnv1a_hash::fnv1a_hash_str_16_xor(&module_name);
        let mut guard = MODULE_LISTS.write().unwrap();
        loop {
            if let Some(module) = guard.get(&hash_value) {
                if *module == module_name {
                    panic!("run_error has already registered module {}.", module_name);
                } else {
                    hash_value = hash_value.wrapping_add(1);
                }
            } else {
                // not found.
                break;
            }
        }
        guard.insert(hash_value, module_name);
        Self(hash_value)
    }
}
