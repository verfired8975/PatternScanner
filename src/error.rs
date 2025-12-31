use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("process yok ortada: {0}")]
    ProcessNotFound(String),

    #[error("process acilmiyor, admin mi calistirdin?: {0}")]
    OpenProcessFailed(String),

    #[error("burasi okunamiyor moruk 0x{0:X}")]
    ReadMemoryFailed(usize),

    #[error("pattern yanlis yazilmis: {0}")]
    InvalidPattern(String),

    #[error("moduller enumlanmiyor")]
    EnumModulesFailed,

    #[error("modul bulunamadi: {0}")]
    ModuleNotFound(String),

    #[error("windows sikiyet etti: {0}")]
    WindowsApi(#[from] windows::core::Error),
}
