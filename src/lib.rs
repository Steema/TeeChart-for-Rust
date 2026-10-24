//! Rust wrapper for the `TeeChartRender` function of the TeeChart native library.
//!
//! The call follows the same C ABI used by the R (`.C`) and Python (`ctypes`)
//! packages: every argument is passed by pointer.

use std::ffi::{c_char, c_double, c_int, CString};
use std::fmt;
use std::path::PathBuf;
use std::sync::OnceLock;

use libloading::Library;

#[cfg(target_os = "windows")]
const LIB_DIR: &str = "win64";
#[cfg(target_os = "windows")]
const LIB_FILE: &str = "teechart.dll";

#[cfg(target_os = "linux")]
const LIB_DIR: &str = "linux64";
#[cfg(target_os = "linux")]
const LIB_FILE: &str = "libteechart.so";

#[cfg(target_os = "macos")]
const LIB_DIR: &str = "macos";
#[cfg(target_os = "macos")]
const LIB_FILE: &str = "libteechart.dylib";

#[derive(Debug)]
pub enum Error {
    /// The native library could not be found or loaded.
    Load(String),
    /// `TeeChartRender` was not found in the library.
    Symbol(String),
    /// A string contains an interior NUL byte.
    InvalidText(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Load(m) => write!(f, "cannot load TeeChart library: {m}"),
            Error::Symbol(m) => write!(f, "TeeChartRender not found: {m}"),
            Error::InvalidText(s) => write!(f, "text contains NUL: {s:?}"),
        }
    }
}

impl std::error::Error for Error {}

type RenderFn = unsafe extern "C" fn(
    *mut c_int, *mut c_double,        // nx, x
    *mut c_int, *mut c_double,        // ny, y
    *mut c_int, *mut c_double,        // nz, z
    *mut c_int, *mut *mut c_char,     // ntexts, texts
    *mut *mut c_char,                 // output type
    *mut c_int, *mut c_int,           // width, height
    *mut *mut c_char,                 // style
    *mut *mut c_char,                 // title
    *mut *mut c_char,                 // footer
    *mut c_int, *mut c_int, *mut c_int, // horiz, commander, opengl
);

static LIB: OnceLock<Result<Library, String>> = OnceLock::new();

/// Candidate locations, in priority order.
fn candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(p) = std::env::var_os("TEECHART_LIB") {
        v.push(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            v.push(dir.join(LIB_FILE));
        }
    }
    v.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("libs")
            .join(LIB_DIR)
            .join(LIB_FILE),
    );
    v
}

fn library() -> Result<&'static Library, Error> {
    LIB.get_or_init(|| {
        let mut last = String::from("no candidate paths");
        for path in candidates() {
            if !path.is_file() {
                last = format!("not found: {}", path.display());
                continue;
            }
            // Safety: loading a native library runs its initialization code.
            match unsafe { Library::new(&path) } {
                Ok(lib) => return Ok(lib),
                Err(e) => last = format!("{}: {e}", path.display()),
            }
        }
        Err(last)
    })
    .as_ref()
    .map_err(|e| Error::Load(e.clone()))
}

fn cstr(s: &str) -> Result<CString, Error> {
    CString::new(s).map_err(|_| Error::InvalidText(s.to_string()))
}

/// A chart definition. Build it with the chained methods and call [`Chart::render`].
#[derive(Debug, Clone)]
pub struct Chart {
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
    texts: Vec<String>,
    output_type: String,
    width: i32,
    height: i32,
    style: String,
    title: String,
    footer: String,
    horiz: bool,
    commander: bool,
    opengl: bool,
}

impl Default for Chart {
    fn default() -> Self {
        Chart {
            x: vec![],
            y: vec![],
            z: vec![],
            texts: vec![],
            output_type: "modal".into(),
            width: 800,
            height: 500,
            style: String::new(),
            title: String::new(),
            footer: String::new(),
            horiz: false,
            commander: false,
            opengl: false,
        }
    }
}

impl Chart {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn x(mut self, v: &[f64]) -> Self { self.x = v.to_vec(); self }
    pub fn y(mut self, v: &[f64]) -> Self { self.y = v.to_vec(); self }
    pub fn z(mut self, v: &[f64]) -> Self { self.z = v.to_vec(); self }
    pub fn texts(mut self, v: &[&str]) -> Self {
        self.texts = v.iter().map(|s| s.to_string()).collect();
        self
    }
    /// e.g. `"modal"`; use the same values as the R and Python packages.
    pub fn output_type(mut self, s: &str) -> Self { self.output_type = s.into(); self }
    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    pub fn style(mut self, s: &str) -> Self { self.style = s.into(); self }
    pub fn title(mut self, s: &str) -> Self { self.title = s.into(); self }
    pub fn footer(mut self, s: &str) -> Self { self.footer = s.into(); self }
    pub fn horiz(mut self, b: bool) -> Self { self.horiz = b; self }
    pub fn commander(mut self, b: bool) -> Self { self.commander = b; self }
    pub fn opengl(mut self, b: bool) -> Self { self.opengl = b; self }

    /// Calls `TeeChartRender` in the native library.
    pub fn render(&self) -> Result<(), Error> {
        let lib = library()?;
        let render = unsafe { lib.get::<RenderFn>(b"TeeChartRender\0") }
            .map_err(|e| Error::Symbol(e.to_string()))?;

        // The C ABI takes mutable pointers (R's .C style), so work on copies.
        let mut x = self.x.clone();
        let mut y = self.y.clone();
        let mut z = self.z.clone();

        let text_owned: Vec<CString> =
            self.texts.iter().map(|s| cstr(s)).collect::<Result<_, _>>()?;
        let mut text_ptrs: Vec<*mut c_char> =
            text_owned.iter().map(|c| c.as_ptr() as *mut c_char).collect();

        let output_c = cstr(&self.output_type)?;
        let style_c = cstr(&self.style)?;
        let title_c = cstr(&self.title)?;
        let footer_c = cstr(&self.footer)?;
        let mut output_p = output_c.as_ptr() as *mut c_char;
        let mut style_p = style_c.as_ptr() as *mut c_char;
        let mut title_p = title_c.as_ptr() as *mut c_char;
        let mut footer_p = footer_c.as_ptr() as *mut c_char;

        let (mut nx, mut ny, mut nz, mut nt) = (
            x.len() as c_int,
            y.len() as c_int,
            z.len() as c_int,
            text_ptrs.len() as c_int,
        );
        let (mut w, mut h) = (self.width as c_int, self.height as c_int);
        let (mut horiz, mut commander, mut opengl) = (
            self.horiz as c_int,
            self.commander as c_int,
            self.opengl as c_int,
        );

        unsafe {
            render(
                &mut nx, x.as_mut_ptr(),
                &mut ny, y.as_mut_ptr(),
                &mut nz, z.as_mut_ptr(),
                &mut nt, text_ptrs.as_mut_ptr(),
                &mut output_p,
                &mut w, &mut h,
                &mut style_p, &mut title_p, &mut footer_p,
                &mut horiz, &mut commander, &mut opengl,
            );
        }
        Ok(())
    }
  }
