use crate::cape_data_traits::*;
use crate::C;
use crate::cape_result_value::*;
use std::fmt;

/// Default ICapeString implementation
///
/// For COBIA, strings go over the pipeline
///  as null terminated. For Windows, COBIA requires UTF-16 encoding.
///
/// Rust uses non-null-terminated strings that are UTF-8 encoded.
///
/// Because this translation is always required, there is no 
/// read-only string implementation that refers to a string slice.
///
/// This implementation uses a `Vec<u16>` to store the string data.
/// In case the string is empty, the vector may remain empty, so
/// care must be taken to add a null terminated string in this scenario.
///

#[derive(Debug,Clone)]
pub struct CapeStringImpl {
	data: Vec<C::CapeCharacter>, //we allow the vec to remain empty (no trailing 0) if uninitialized. This implies some checks need to be done for the empty case.
}

impl CapeStringImpl {

	const EMPTY_STRING : C::CapeCharacter = 0;

	///Default constructor
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::new();
	/// assert_eq!(s.as_string(),"");
	/// ```
	pub fn new() -> Self {
		CapeStringImpl {
			data: Vec::new(),
		}
	}
	///Construct from string
	///
	/// # Arguments
	///
	/// * `s` - A string slice to be converted to a CapeStringImpl
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// ```
	pub fn from_string<T:AsRef<str>>(s: T) -> Self {
		let mut cs = CapeStringImpl::new();
		let s=s.as_ref();
		cs.data.reserve(s.len() + 1);
		for c in s.encode_utf16() { //todo, change to s.encode_utf16().collect_into(&cs.data);
			cs.data.push(c);
		}
		cs.data.push(0);
		cs
	}
	///Construct from raw data
	///
	/// # Arguments
	///
	/// * `data` - A pointer to the string data
	/// * `size` - The size of the string data
	///
	pub unsafe fn from_raw_data(data: *const C::CapeCharacter, size: C::CapeSize) -> Self {
		let mut cs = CapeStringImpl::new();
		cs.data.reserve(size as usize + 1);
		let slice = unsafe { std::slice::from_raw_parts(data, size as usize) };
		cs.data.extend_from_slice(&slice);
		cs.data.push(0);
		cs
	}

	///Return as string
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s.as_string(),"idealGasEnthalpy");
	/// ```
	pub fn as_string(&self) -> String {
		if self.data.is_empty() {
			String::new()
		} else {
			let len = self.data.len() - 1;
			String::from_utf16_lossy(&self.data[..len])
		}
	}

	///Set string
	///
	/// # Arguments
	///
	/// * `s` - Any CAPE-OPEN string type
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let mut s=cobia::CapeStringImpl::new();
	/// s.set(&cobia::CapeStringImpl::from_string("idealGasEnthalpy"));
	/// assert_eq!(s.as_string(),"idealGasEnthalpy");
	/// ```

	pub fn set<T: CapeStringConstProvider>(&mut self, s: &T) {
		let(ptr,size)=s.as_capechar_const_with_length();
		self.data.clear();
		self.data.reserve(size as usize + 1);
		let slice=unsafe { std::slice::from_raw_parts(ptr, size as usize) };
		self.data.extend_from_slice(&slice);
		self.data.push(0);
	}

	///Set string
	///
	/// # Arguments
	///
	/// * `s` - A string slice to be set
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let mut s=cobia::CapeStringImpl::new();
	/// s.set_string("idealGasEnthalpy");
	/// assert_eq!(s.as_string(),"idealGasEnthalpy");
	/// ```

	pub fn set_string<T: AsRef<str>>(&mut self, s: T) {
		let s = s.as_ref();
		self.data.clear();
		self.data.reserve(s.len() + 1);
		for c in s.encode_utf16() { //todo, change to s.encode_utf16().collect_into(&cs.data);
			self.data.push(c);
		}
		self.data.push(0);
	}

	/// Check empty
	///
	/// Returns true if the string is empty (no data, or only the null terminator).
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::new();
	/// assert_eq!(s.is_empty(),true);
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s.is_empty(),false);
	/// ```
	pub fn is_empty(&self) -> bool {
		self.data.is_empty() || self.data[0] == 0
	}

	extern "C" fn string_get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *const C::CapeCharacter,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut CapeStringImpl;
		let s: &mut CapeStringImpl = unsafe { &mut *p };
		if s.data.is_empty() {
			unsafe {
				*data = &Self::EMPTY_STRING;
				*size = 0;
			}
		} else {
			unsafe {
				*data = s.data.as_ptr() as *const C::CapeCharacter;
				*size = s.data.len() as C::CapeSize - 1; //exclude null terminator
			}
		}
	}

	extern "C" fn string_set(
		me: *mut ::std::os::raw::c_void,
		data: *const C::CapeCharacter,
		size: C::CapeSize,
	) -> C::CapeResult {
		let p = me as *mut CapeStringImpl;
		let s: &mut CapeStringImpl = unsafe { &mut *p };
		s.data.clear();
		if size > 0 {
			let slice=unsafe { std::slice::from_raw_parts(data, size as usize) };
			s.data.reserve(slice.len() + 1);
			s.data.extend_from_slice(&slice);
			s.data.push(0); //null terminator
		}
		COBIAERR_NOERROR
	}

	const CAPE_STRING_VTABLE: C::ICapeString_VTable = C::ICapeString_VTable {
		get: Some(CapeStringImpl::string_get),
		set: Some(CapeStringImpl::string_set),
	};

	/// Convert character to lower case
	///
	/// # Arguments
	///
	/// * `c` - A UTF-16 character to convert to lower case
	///

	pub(crate) fn to_lower_case(c: C::CapeCharacter) -> C::CapeCharacter {
		#![allow(non_contiguous_range_endpoints)]
		//https://www.ibm.com/docs/en/i/7.5?topic=tables-unicode-uppercase-lowercase-conversion-mapping-table
		//note that all characters live in the U+0000 to U+D7FF and U+E000 to U+FFFF range 
		// https://en.wikipedia.org/wiki/UTF-16
		//so that there is no need to check for the surrogate pair range (or for 4-byte UTF-16 characters)
		match c {
			65..90 | 192..214 | 216..222 | 913..929 | 931..939 | 1040..1071 | 65313..65338 => c+32,
			256 | 258 | 260 | 262 | 264 | 266 | 268 | 270 | 272 | 274 | 276 | 278 | 280 | 282 | 284 | 286 | 288 | 290 | 292 | 294 | 296 | 298 | 300 | 302 | 306 | 308 | 310 | 313 | 315 | 317 | 319 | 321 | 323 | 325 | 327 | 330 | 332 | 334 | 336 | 338 | 340 | 342 | 344 | 346 | 348 | 350 | 352 | 354 | 356 | 358 | 360 | 362 | 364 | 366 | 368 | 370 | 372 | 374 | 377 | 379 | 381 | 386 | 388 | 391 | 395 | 401 | 408 | 416 | 418 | 420 | 423 | 428 | 431 | 435 | 437 | 440 | 444 | 453 | 456 | 459 | 461 | 463 | 465 | 467 | 469 | 471 | 473 | 475 | 478 | 480 | 482 | 484 | 486 | 488 | 490 | 492 | 494 | 500 | 506 | 508 | 510 | 512 | 514 | 516 | 518 | 520 | 522 | 524 | 526 | 528 | 530 | 532 | 534 | 994 | 996 | 998 | 1000 | 1002 | 1004 | 1006 | 1120 | 1122 | 1124 | 1126 | 1128 | 1130 | 1132 | 1134 | 1136 | 1138 | 1140 | 1142 | 1144 | 1146 | 1148 | 1150 | 1152 | 1168 | 1170 | 1172 | 1174 | 1176 | 1178 | 1180 | 1182 | 1184 | 1186 | 1188 | 1190 | 1192 | 1194 | 1196 | 1198 | 1200 | 1202 | 1204 | 1206 | 1208 | 1210 | 1212 | 1214 | 1217 | 1219 | 1223 | 1227 | 1232 | 1234 | 1236 | 1238 | 1240 | 1242 | 1244 | 1246 | 1248 | 1250 | 1252 | 1254 | 1256 | 1258 | 1262 | 1264 | 1266 | 1268 | 1272 | 7680 | 7682 | 7684 | 7686 | 7688 | 7690 | 7692 | 7694 | 7696 | 7698 | 7700 | 7702 | 7704 | 7706 | 7708 | 7710 | 7712 | 7714 | 7716 | 7718 | 7720 | 7722 | 7724 | 7726 | 7728 | 7730 | 7732 | 7734 | 7736 | 7738 | 7740 | 7742 | 7744 | 7746 | 7748 | 7750 | 7752 | 7754 | 7756 | 7758 | 7760 | 7762 | 7764 | 7766 | 7768 | 7770 | 7772 | 7774 | 7776 | 7778 | 7780 | 7782 | 7784 | 7786 | 7788 | 7790 | 7792 | 7794 | 7796 | 7798 | 7800 | 7802 | 7804 | 7806 | 7808 | 7810 | 7812 | 7814 | 7816 | 7818 | 7820 | 7822 | 7824 | 7826 | 7828 | 7840 | 7842 | 7844 | 7846 | 7848 | 7850 | 7852 | 7854 | 7856 | 7858 | 7860 | 7862 | 7864 | 7866 | 7868 | 7870 | 7872 | 7874 | 7876 | 7878 | 7880 | 7882 | 7884 | 7886 | 7888 | 7890 | 7892 | 7894 | 7896 | 7898 | 7900 | 7902 | 7904 | 7906 | 7908 | 7910 | 7912 | 7914 | 7916 | 7918 | 7920 | 7922 | 7924 | 7926 | 7928 => c+1,
			304 => c-199,
			376 => c-121,
			385 => c+210,
			390 => c+206,
			394 | 403 => c+205,
			398..399 => c+202,
			400 => c+203,
			404 => c+207,
			406 | 412 => c+211,
			407 => c+209,
			413 => c+213,
			415 => c+214,
			425 | 430 => c+218,
			433..434 => c+217,
			439 => c+219,
			452 | 455 | 458 | 497 => c+2,
			902 => c+38,
			904..906 => c+37,
			908 => c+64,
			910..911 => c+63,
			1025..1036 | 1038..1039 => c+80,
			1329..1366 | 4256..4293 => c+48,
			7944..7951 | 7960..7965 | 7976..7983 | 7992..7999 | 8008..8013 | 8025 | 8027 | 8029 | 8031 | 8040..8047 | 8072..8079 | 8088..8095 | 8104..8111 | 8120..8121 | 8152..8153 | 8168..8169 => c-8,
			9398..9423 => c+26,
			_ => c
		}
	}

	/// Convert character to upper case
	///
	/// # Arguments
	///
	/// * `c` - A UTF-16 character to convert to upper case
	///

	pub(crate) fn to_upper_case(c: C::CapeCharacter) -> C::CapeCharacter {
		#![allow(non_contiguous_range_endpoints)]
		//coded as invers of to_lower_case, with exception of 
		// 0130	0069	LATIN CAPITAL LETTER I WITH DOT ABOVE	LATIN SMALL LETTER I
		// because latin small letter I maps to latin capital letter I without dot
		//https://www.ibm.com/docs/en/i/7.5?topic=tables-unicode-uppercase-lowercase-conversion-mapping-table
		match c {
			97..122 | 224..246 | 248..254 | 945..961 | 963..971 | 1072..1103 | 65345..65370 => c-32,
			257 | 259 | 261 | 263 | 265 | 267 | 269 | 271 | 273 | 275 | 277 | 279 | 281 | 283 | 285 | 287 | 289 | 291 | 293 | 295 | 297 | 299 | 301 | 303 | 307 | 309 | 311 | 314 | 316 | 318 | 320 | 322 | 324 | 326 | 328 | 331 | 333 | 335 | 337 | 339 | 341 | 343 | 345 | 347 | 349 | 351 | 353 | 355 | 357 | 359 | 361 | 363 | 365 | 367 | 369 | 371 | 373 | 375 | 378 | 380 | 382 | 387 | 389 | 392 | 396 | 402 | 409 | 417 | 419 | 421 | 424 | 429 | 432 | 436 | 438 | 441 | 445 | 462 | 464 | 466 | 468 | 470 | 472 | 474 | 476 | 479 | 481 | 483 | 485 | 487 | 489 | 491 | 493 | 495 | 501 | 507 | 509 | 511 | 513 | 515 | 517 | 519 | 521 | 523 | 525 | 527 | 529 | 531 | 533 | 535 | 995 | 997 | 999 | 1001 | 1003 | 1005 | 1007 | 1121 | 1123 | 1125 | 1127 | 1129 | 1131 | 1133 | 1135 | 1137 | 1139 | 1141 | 1143 | 1145 | 1147 | 1149 | 1151 | 1153 | 1169 | 1171 | 1173 | 1175 | 1177 | 1179 | 1181 | 1183 | 1185 | 1187 | 1189 | 1191 | 1193 | 1195 | 1197 | 1199 | 1201 | 1203 | 1205 | 1207 | 1209 | 1211 | 1213 | 1215 | 1218 | 1220 | 1224 | 1228 | 1233 | 1235 | 1237 | 1239 | 1241 | 1243 | 1245 | 1247 | 1249 | 1251 | 1253 | 1255 | 1257 | 1259 | 1263 | 1265 | 1267 | 1269 | 1273 | 7681 | 7683 | 7685 | 7687 | 7689 | 7691 | 7693 | 7695 | 7697 | 7699 | 7701 | 7703 | 7705 | 7707 | 7709 | 7711 | 7713 | 7715 | 7717 | 7719 | 7721 | 7723 | 7725 | 7727 | 7729 | 7731 | 7733 | 7735 | 7737 | 7739 | 7741 | 7743 | 7745 | 7747 | 7749 | 7751 | 7753 | 7755 | 7757 | 7759 | 7761 | 7763 | 7765 | 7767 | 7769 | 7771 | 7773 | 7775 | 7777 | 7779 | 7781 | 7783 | 7785 | 7787 | 7789 | 7791 | 7793 | 7795 | 7797 | 7799 | 7801 | 7803 | 7805 | 7807 | 7809 | 7811 | 7813 | 7815 | 7817 | 7819 | 7821 | 7823 | 7825 | 7827 | 7829 | 7841 | 7843 | 7845 | 7847 | 7849 | 7851 | 7853 | 7855 | 7857 | 7859 | 7861 | 7863 | 7865 | 7867 | 7869 | 7871 | 7873 | 7875 | 7877 | 7879 | 7881 | 7883 | 7885 | 7887 | 7889 | 7891 | 7893 | 7895 | 7897 | 7899 | 7901 | 7903 | 7905 | 7907 | 7909 | 7911 | 7913 | 7915 | 7917 | 7919 | 7921 | 7923 | 7925 | 7927 | 7929 => c-1,
			255 => c+121,
			595 => c-210,
			596 => c-206,
			599 | 608 => c-205,
			600..601 => c-202,
			603 => c-203,
			611 => c-207,
			617 | 623 => c-211,
			616 => c-209,
			626 => c-213,
			629 => c-214,
			643 | 648 => c-218,
			650..651 => c-217,
			658 => c-219,
			454 | 457 | 460 | 499 => c-2,
			940 => c-38,
			941..943 => c-37,
			972 => c-64,
			973..974 => c-63,
			1105..1116 | 1118..1119 => c-80,
			1377..1414 | 4304..4341 => c-48,
			7936..7943 | 7952..7957 | 7968..7975 | 7984..7991 | 8000..8005 | 8017 | 8019 | 8021 | 8023 | 8032..8039 | 8064..8071 | 8080..8087 | 8096..8103 | 8112..8113 | 8144..8145 | 8160..8161 => c+8,
			9424..9449 => c-26,
			_ => c
		}
	}

	/// Convert to lower case.
	///
	/// Most CAPE-OPEN string comparisons are case insensitive. By allowing converson to lower
	/// case, we can use this in conjuntion with Eq, PartialEq, Hash, etc to make lookup tables
	/// and comparisons without the need to convert to Rust's utf-8 encoded string (on Windows,
	/// CapeString is UTF-16 encoded and null terminated).
	///
	/// Note that this is a simple conversion of the first character of each code point.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s.to_lowercase(),cobia::CapeStringImpl::from_string("idealgasenthalpy"));
	/// ```

	pub fn to_lowercase(&self) -> Self {
		let mut cs = CapeStringImpl::new();
		cs.data.reserve(self.data.len());
		for c in self.data.iter() {
			cs.data.push(Self::to_lower_case(*c));
		}
		cs
	}

	/// Convert to upper case.
	///
	/// Most CAPE-OPEN string comparisons are case insensitive. By allowing converson to upper
	/// case, we can use this in conjuntion with Eq, PartialEq, Hash, etc to make lookup tables
	/// and comparisons without the need to convert to Rust's utf-8 encoded string (on Windows,
	/// CapeString is UTF-16 encoded and null terminated).
	///
	/// Note that this is a simple conversion of the first character of each code point.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s.to_uppercase(),cobia::CapeStringImpl::from_string("IDEALGASENTHALPY"));
	/// ```

	pub fn to_uppercase(&self) -> Self {
		let mut cs = CapeStringImpl::new();
		cs.data.reserve(self.data.len());
		for c in self.data.iter() {
			cs.data.push(Self::to_upper_case(*c));
		}
		cs
	}

	/// Case insentitive comparison
	/// 
	/// # Arguments
	///	
	/// * `other` - The other string to compare to
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
	/// assert_eq!(s1.eq_ignore_case(&s2),true);
	/// ```
	pub fn eq_ignore_case<T:CapeStringConstProvider>(&self, other: &T) -> bool {
		let (ptr,len)=other.as_capechar_const_with_length();
		let mut self_len:usize=self.data.len();
		if self_len>0 {self_len-=1;}
		let len=len as usize;
		if self_len == len {
			let mut ptr1=ptr;
			for i in 0..len {
				if CapeStringImpl::to_lower_case(unsafe { *ptr1 }) != CapeStringImpl::to_lower_case(self.data[i]) {
					return false;
				}
				ptr1=unsafe { ptr1.add(1) };
			}
			return true;			
		}
		false
	}

}

impl CapeStringConstProvider for CapeStringImpl {
	///Return as CapeCharacter const pointer with length
	///
	/// The caller must ensure that the lifetime of the CapeStringImpl
	/// is longer than the pointer returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy"); //must remain in scope....
	/// let (ptr,len)=s.as_capechar_const_with_length(); ///... while ptr is used
	/// assert_eq!(len,16);
	/// ```
	fn as_capechar_const_with_length(&self) -> (*const C::CapeCharacter, C::CapeSize) {
		if self.data.is_empty() {
			("\0\0".as_ptr() as *const C::CapeCharacter, 0) //need just the terminating null
		} else {
			(self.data.as_ptr() as *const C::CapeCharacter, (self.data.len() - 1) as C::CapeSize) //length without the terminating null
		}
	}
	///Return as CapeCharacter const pointer with length
	///
	/// The caller must ensure that the lifetime of the CapeStringImpl
	/// is longer than the pointer returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy"); //must remain in scope....
	/// let ptr=s.as_capechar_const(); ///... while ptr is used
	/// assert_eq!(unsafe{*ptr},'i' as u16);
	/// ```
	fn as_capechar_const(&self) -> *const C::CapeCharacter {
		if self.data.is_empty() {
			"\0\0".as_ptr() as *const C::CapeCharacter //need just the terminating null
		} else {
			self.data.as_ptr() as *const C::CapeCharacter
		}
	}
}

impl fmt::Display for CapeStringImpl {
	/// Formats the CapeStringImpl error using the given formatter.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(format!("{}",s),"idealGasEnthalpy");
	/// ```
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::new();
	/// assert_eq!(format!("{}",s),"");
	/// ```
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.data.is_empty() {
			write!(f, "")
		} else {
			let len = self.data.len() - 1;
			write!(f, "{}", String::from_utf16_lossy(&self.data[..len]))
		}
	}
}

impl CapeStringProviderIn for CapeStringImpl {
	fn as_cape_string_in(&self) -> C::ICapeString {
		C::ICapeString {
			vTbl:(&CapeStringImpl::CAPE_STRING_VTABLE as *const C::ICapeString_VTable).cast_mut(),
			me:(self as *const CapeStringImpl).cast_mut() as *mut ::std::os::raw::c_void
		}
	}
}

impl CapeStringProviderOut for CapeStringImpl {
	fn as_cape_string_out(&mut self) -> C::ICapeString {
		C::ICapeString {
			vTbl:(&CapeStringImpl::CAPE_STRING_VTABLE as *const C::ICapeString_VTable).cast_mut(),
			me:(self as *const CapeStringImpl).cast_mut() as *mut ::std::os::raw::c_void
		}
	}
}

impl<T:CapeStringConstProvider> PartialEq<T> for CapeStringImpl {

	/// Case sensitive comparison
	///
	/// # Arguments
	/// * other - The other string to compare to
	///
	/// # Return
	/// * true if the strings are equal, false otherwise.
	///
	/// # Examples
	/// 
	/// ```
	/// use cobia;
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// let s2=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s1==s2,true);
	/// let s3=cobia::CapeStringImpl::from_string("IdealGasEnthalpy");
	/// assert_eq!(s1==s3,false);
	/// ```
	fn eq(&self, other: &T) -> bool {
		let (ptr,len)=other.as_capechar_const_with_length();
		let mut self_len:usize=self.data.len();
		if self_len>0 {self_len-=1;}
		let len=len as usize;
		if self_len == len {
			let mut ptr1=ptr;
			for i in 0..len {
				if unsafe { *ptr1 } != self.data[i] {
					return false;
				}
				ptr1=unsafe { ptr1.add(1) };
			}
			return true;			
		}
		false
	}
}

impl Eq for CapeStringImpl {
}

impl std::hash::Hash for CapeStringImpl {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.data.hash(state);
	}
}