use crate::C;

pub type CapeWindowId = crate::C::HWND;

pub fn CapeWindowIdToRaw(hwnd: CapeWindowId) -> C::CapeWindowId {
	hwnd as C::CapeWindowId
}

pub fn CapeWindowIdFromRaw(hwnd: C::CapeWindowId) -> CapeWindowId {
	hwnd as CapeWindowId
}

