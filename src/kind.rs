pub struct RErrorKind(u16);

impl RErrorKind {
    #[inline]
    const fn new(code: u8) -> Self {
        Self(Self::encode_code(code, false))
    }

    #[inline]
    pub const fn new_custom(code: u8) -> Self {
        Self(Self::encode_code(code, true))
    }

    #[inline]
    const fn encode_code(code: u8, is_custom: bool) -> u16 {
        if is_custom {
            ((code as u16) << 4) | 0x8000
        } else {
            ((code as u16) << 4) | 0x0000
        }
    }

    #[inline]
    const fn decode_code(code: u16) -> (u8, bool) {
        let is_custom = (code & 0x8000) != 0;
        (((code >> 4) as u8), is_custom)
    }

    #[inline]
    pub(crate) const fn get_encode_code(&self) -> u16 {
        self.0
    }

    #[inline]
    pub(crate) const fn from_encode_code(code: u16) -> Self {
        // assert_eq!(code % 4, 0);
        Self(code)
    }
}

impl std::fmt::Debug for RErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for RErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (code, is_custom) = Self::decode_code(self.0);
        if is_custom {
            write!(f, "MODULE ERROR ({})", code)
        } else {
            match BuildInErrKind::try_from(code) {
                Err(_) => {
                    write!(f, "UNKNOWN ERROR ({})", code)
                }
                Ok(code) => {
                    write!(f, "{} ({})", code, code as u8)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum BuildInErrKind {
    NotAllowed = 0, /* Not allowed */
    NotFound,       /* Not found */
    NotReady,       /* Not Ready */
    AccessDenied,   /* Access denied */
    InternalErr,    /* Internal error */
    AlreadyExist,   /* Already exist */
    InvalidValue,   /* Invalid argument */
    NotAvailable,   /* Not available */
    InUse,          /* Already in use */
    Unreachable,    /* Unreachable */
    NoMemory,       /* No memory */
    Deficiency,     /* Deficiency */
    TimeOut,        /* Time out */
    Interrupted,    /* Interrupted */
    TooMany,        /* Too many */
    Changed,        /* Something changed */
    NetErr,         /* Network error */
    IOErr,          /* IO error */
    DeviceErr,      /* Device error */
    OSErr,          /* OS error */
    FFIErr,         /* FFI error */
    RemoteErr,      /* Remote node error */
    NumOfKind,
}

impl std::fmt::Display for BuildInErrKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl TryFrom<u8> for BuildInErrKind {
    type Error = RErrorKind;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value >= Self::NumOfKind as u8 {
            return Err(INVALID_VALUE);
        }
        unsafe { Ok(std::mem::transmute::<u8, BuildInErrKind>(value)) }
    }
}

impl BuildInErrKind {
    const DESCS: [&'static str; Self::NumOfKind as usize] = [
        "NOT ALLOWED",
        "NOT FOUND",
        "NOT ALREADY",
        "ACCESS DENIED",
        "INTERNAL ERROR",
        "ALREADY EXIST",
        "INVALID ARGUMENT",
        "NOT AVAILABLE",
        "IN USE",
        "UNREACHABLE",
        "NO MEMORY",
        "DEFICIENCY",
        "TIME OUT",
        "INTERRUPTED",
        "TOO MANY/MUCH",
        "SOMETHING CHANGED",
        "NETWORK ERROR",
        "IO ERROR",
        "DEVICE ERROR",
        "OS ERROR",
        "FFI ERROR",
        "REMOTE ERROR",
    ];
    fn description(&self) -> &'static str {
        Self::DESCS[(*self) as usize]
    }
}

pub const NOT_ALLOWED: RErrorKind = RErrorKind::new(BuildInErrKind::NotAllowed as u8);
pub const NOT_FOUND: RErrorKind = RErrorKind::new(BuildInErrKind::NotFound as u8);
pub const NOT_READY: RErrorKind = RErrorKind::new(BuildInErrKind::NotReady as u8);
pub const ACCESS_DENIED: RErrorKind = RErrorKind::new(BuildInErrKind::AccessDenied as u8);
pub const INTERNAL_ERR: RErrorKind = RErrorKind::new(BuildInErrKind::InternalErr as u8);
pub const ALREADY_EXIST: RErrorKind = RErrorKind::new(BuildInErrKind::AlreadyExist as u8);
pub const INVALID_VALUE: RErrorKind = RErrorKind::new(BuildInErrKind::InvalidValue as u8);
pub const NOT_AVAILABLE: RErrorKind = RErrorKind::new(BuildInErrKind::NotAvailable as u8);
pub const IN_USE: RErrorKind = RErrorKind::new(BuildInErrKind::InUse as u8);
pub const UNREACHABLE: RErrorKind = RErrorKind::new(BuildInErrKind::Unreachable as u8);
pub const NO_MEMORY: RErrorKind = RErrorKind::new(BuildInErrKind::NoMemory as u8);
pub const DEFICIENCY: RErrorKind = RErrorKind::new(BuildInErrKind::Deficiency as u8);
pub const TIMED_OUT: RErrorKind = RErrorKind::new(BuildInErrKind::TimeOut as u8);
pub const INTERRUPTED: RErrorKind = RErrorKind::new(BuildInErrKind::Interrupted as u8);
pub const TOO_MANY: RErrorKind = RErrorKind::new(BuildInErrKind::TooMany as u8);
pub const CHANGED: RErrorKind = RErrorKind::new(BuildInErrKind::Changed as u8);
pub const NET_ERR: RErrorKind = RErrorKind::new(BuildInErrKind::NetErr as u8);
pub const IOERR: RErrorKind = RErrorKind::new(BuildInErrKind::IOErr as u8);
pub const DEVICE_ERR: RErrorKind = RErrorKind::new(BuildInErrKind::DeviceErr as u8);
pub const OSERR: RErrorKind = RErrorKind::new(BuildInErrKind::OSErr as u8);
pub const FFIERR: RErrorKind = RErrorKind::new(BuildInErrKind::FFIErr as u8);
pub const REMOTE_ERR: RErrorKind = RErrorKind::new(BuildInErrKind::RemoteErr as u8);
