use libc::{c_char, c_int, c_longlong, size_t};
use std::ffi::{CStr, CString};
use std::ptr;
use thiserror::Error;

#[repr(C)]
struct SPFreshStatus {
    code: i32,
    message: *const c_char,
}

type Handle = *mut core::ffi::c_void;

extern "C" {
    fn spfresh_open(
        index_dir: *const c_char,
        dim: c_int,
        params: *const c_char,
        out: *mut Handle,
    ) -> SPFreshStatus;
    fn spfresh_close(handle: Handle);
    fn spfresh_add(
        handle: Handle,
        vectors: *const f32,
        n: size_t,
        ids: *const c_longlong,
    ) -> SPFreshStatus;
    fn spfresh_search(
        handle: Handle,
        query: *const f32,
        topk: c_int,
        out_ids: *mut c_longlong,
        out_scores: *mut f32,
    ) -> SPFreshStatus;

    #[allow(dead_code)] // 👈 ปิดเตือนถ้ายังไม่เรียกใช้งาน
    fn spfresh_save(handle: Handle) -> SPFreshStatus;
}

#[derive(Debug, Error)]
pub enum SpfreshError {
    #[error("FFI error: {0}")]
    Ffi(String),
    #[error("invalid parameter: {0}")]
    InvalidParam(&'static str),
}

fn into_result(st: SPFreshStatus) -> Result<(), SpfreshError> {
    if st.code == 0 {
        return Ok(());
    }
    let msg = unsafe {
        if st.message.is_null() {
            "unknown error".into()
        } else {
            CStr::from_ptr(st.message).to_string_lossy().into_owned()
        }
    };
    Err(SpfreshError::Ffi(msg))
}

pub struct Spfresh {
    h: Handle,
    dim: usize,
}

unsafe impl Send for Spfresh {}
unsafe impl Sync for Spfresh {}

impl Spfresh {
    pub fn open(index_dir: &str, dim: usize, params: &str) -> Result<Self, SpfreshError> {
        if dim == 0 {
            return Err(SpfreshError::InvalidParam("dim == 0"));
        }
        let idx = CString::new(index_dir).map_err(|_| SpfreshError::InvalidParam("index_dir contains NUL"))?;
        let par = CString::new(params).map_err(|_| SpfreshError::InvalidParam("params contains NUL"))?;
        let mut h: Handle = ptr::null_mut();
        let st = unsafe { spfresh_open(idx.as_ptr(), dim as c_int, par.as_ptr(), &mut h) };
        into_result(st)?;
        Ok(Self { h, dim })
    }

    pub fn add_batch(&self, vectors: &[f32], ids: Option<&[i64]>) -> Result<(), SpfreshError> {
        if self.dim == 0 {
            return Err(SpfreshError::InvalidParam("self.dim == 0"));
        }
        let n = vectors.len() / self.dim;
        if vectors.len() != n * self.dim {
            return Err(SpfreshError::InvalidParam("vectors buffer must be contiguous [n*dim]"));
        }
        if let Some(ids_slice) = ids {
            if ids_slice.len() != n {
                return Err(SpfreshError::InvalidParam("ids length != number of vectors"));
            }
        }
        if n == 0 {
            return Ok(());
        }

        let ids_ptr = ids.map(|v| v.as_ptr()).unwrap_or(ptr::null());
        let st = unsafe { spfresh_add(self.h, vectors.as_ptr(), n as size_t, ids_ptr) };
        into_result(st)
    }

    pub fn search(&self, query: &[f32], topk: usize) -> Result<(Vec<i64>, Vec<f32>), SpfreshError> {
        if self.dim == 0 {
            return Err(SpfreshError::InvalidParam("self.dim == 0"));
        }
        if query.len() != self.dim {
            return Err(SpfreshError::InvalidParam("query dim mismatch"));
        }
        if topk == 0 {
            return Ok((Vec::new(), Vec::new()));
        }

        let mut ids = vec![0i64; topk];
        let mut scores = vec![0f32; topk];
        let st = unsafe {
            spfresh_search(
                self.h,
                query.as_ptr(),
                topk as c_int,
                ids.as_mut_ptr(),
                scores.as_mut_ptr(),
            )
        };
        into_result(st)?;
        Ok((ids, scores))
    }

    #[allow(dead_code)] // 👈 ปิดเตือนถ้ายังไม่เรียกใช้งาน
    pub fn save(&self) -> Result<(), SpfreshError> {
        let st = unsafe { spfresh_save(self.h) };
        into_result(st)
    }
}

impl Drop for Spfresh {
    fn drop(&mut self) {
        if !self.h.is_null() {
            unsafe { spfresh_close(self.h) }
        }
    }
}
