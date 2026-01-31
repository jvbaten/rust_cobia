use crate::C;

pub struct CapeWindowId {
	pub h_wnd : crate::C::HWND
}

unsafe impl Send for CapeWindowId{}
unsafe impl Sync for CapeWindowId{}

pub fn CapeWindowIdToRaw(hwnd: CapeWindowId) -> C::CapeWindowId {
	hwnd.h_wnd
}

pub fn CapeWindowIdFromRaw(hwnd: C::CapeWindowId) -> CapeWindowId {
	CapeWindowId{ h_wnd: hwnd as C::CapeWindowId }
}

