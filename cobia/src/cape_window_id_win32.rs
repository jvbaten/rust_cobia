use crate::C;

#[derive(Default,Clone)]
pub struct CapeWindowId {
	pub h_wnd: crate::C::HWND,
}

unsafe impl Send for CapeWindowId {}
unsafe impl Sync for CapeWindowId {}

/// CapeWindowId represents a window
impl CapeWindowId {

	/// Convert to raw window id
	///
	/// # Return
	/// Raw window id
	pub fn AsRaw(&self) -> CapeWindowId {
		(*self).clone()
	}

	pub fn as_native(&self) -> C::CapeWindowId {
		self.h_wnd
	}

	/// Convert raw window id to CapeWindowId
	///
	/// # Arguments
	/// * `hwnd` - Raw window id
	///
	/// # Return
	/// CapeWindowId
	pub fn FromRaw(hwnd: C::CapeWindowId) -> CapeWindowId {
		CapeWindowId {
			h_wnd: hwnd as C::CapeWindowId,
		}
	}
}
