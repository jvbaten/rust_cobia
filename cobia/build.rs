extern crate bindgen;

use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::{self,File,OpenOptions};
use std::io::{self, LineWriter,Write,BufRead};
use std::path::PathBuf;

//use bindgen::CargoCallbacks;

struct EnumMember {
	name: String,
	value: String,
}

struct EnumDef<'a> {
	name: &'a str,
	prefix: &'a str,
	bitfield: bool,
}

struct EnumInfo {
	comment: String,
	values: Vec<EnumMember>,
}

fn main() {
	// This is the directory where the `c` library is located.
	let cdir_path = PathBuf::from("src/C")
		// Canonicalize the path as `rustc-link-search` requires an absolute
		// path.
		.canonicalize()
		.expect("cannot canonicalize path");

	let out_dir=match std::env::var_os("OUT_DIR") {
		Some(val) => PathBuf::from(val).canonicalize()
			.expect("cannot canonicalize path"),
		None => panic!("Environment variable OUT_DIR not set")
	};

	// This is the path to the `c` headers file.
	let headers_path = cdir_path.join("C.h");
	let headers_path_str = headers_path.to_str().expect("Path is not a valid string");

	// This is the path to the intermediate object file for our library.
	let obj_path = out_dir.join("C.o");

	//add COBIA include folder
	let cobia_include_path = env::var("COBIA_INCLUDE").unwrap();

	println!("cargo:rustc-link-search={}", cobia_include_path);

	// Tell cargo where to look for our C-library
	println!("cargo:rustc-link-search={}", out_dir.to_str().unwrap());

	// Tell cargo to tell rustc to link our `C` library. Cargo will
	// automatically know it must look for a `libC.a` file.
	println!("cargo:rustc-link-lib=CobiaCbinding");

	// Run `clang` to compile the `C.c` file into a `C.o` object file.
	// Unwrap if it is not possible to spawn the process.

    let (clang,clangpp,ar,rc) = match std::env::var_os("CARGO_CFG_WINDOWS") {
		Some(_) => {
			//windows:
			let clang_path = env::var("LIBCLANG_PATH").unwrap();
			(PathBuf::from(clang_path.clone()).join("clang"),
			 PathBuf::from(clang_path.clone()).join("clang++"),
			 PathBuf::from(clang_path.clone()).join("llvm-ar"),
			 Some(PathBuf::from(clang_path).join("llvm-rc")))
		},
		None => {
			//on other platforms we assume clang is in the path
			(PathBuf::from("clang"),
			 PathBuf::from("clang++"),
			 PathBuf::from("ar"),
			 None)
		}
	};
	
	let mut clang_cmd=std::process::Command::new(clang.clone());
	clang_cmd.arg("-c")
		.arg("-I")
		.arg(cobia_include_path.clone());		
	match std::env::var_os("CARGO_CFG_TARGET_ARCH") {
		Some(val) => {
			match std::env::var_os("CARGO_CFG_WINDOWS") {
				Some(_) => {
					let val=val.into_string().unwrap();
					match val.as_str() {
						"x86" => {
							//windows, x86
							clang_cmd.arg("-m32");
						},
						_ => {}
					}
				},
				None => {}
			};
		},
		None => {}
	}
	if !clang_cmd
		.arg("-o")
		.arg(&obj_path)
		.arg(cdir_path.join("C.c"))
		.output()
		.expect("could not spawn `clang`")
		.status
		.success()
	{
		// Panic if the command was not successful.
		panic!("could not compile object file: {:?}",clang_cmd);
	}

	// Run `ar` to generate the `libCobiaCbinding.a` or `CobiaCbinding.lib`
	// file from the `C.o` file.
	// Unwrap if it is not possible to spawn the process.
	let lib_path=match std::env::var_os("CARGO_CFG_WINDOWS") {
		Some(_) => {
			//windows
			out_dir.join("CobiaCbinding.lib")
		},
		None => {
			//other platforms
			out_dir.join("libCobiaCbinding.a")
		}
	};
	if !std::process::Command::new(ar)
		.arg("rcs")
		.arg(lib_path)
		.arg(obj_path)
		.output()
		.expect("could not spawn `ar`")
		.status
		.success()
	{
		// Panic if the command was not successful.
		panic!("could not emit library file");
	}

	// The bindgen::Builder is the main entry point
	// to bindgen, and lets you build up options for
	// the resulting bindings.
	let bindings = bindgen::Builder::default()
		// The input header we would like to generate
		// bindings for.
		.header(headers_path_str)
		//allow CAPE-OPEN stuff only
		.allowlist_file(".*/(?:COBIA|Cape|cape).*")
		//tell it to look here
		.clang_arg("-I".to_owned() + &cobia_include_path)
		// Tell cargo to invalidate the built crate whenever any of the
		// included header files changed.
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		// Finish the builder and generate the bindings.
		.generate()
		// Unwrap the Result and panic on failure.
		.expect("Unable to generate bindings");

	// Write the bindings to the $OUT_DIR/COBIA.rs file.
	let out_path = out_dir.join("c_binding.rs");
	bindings
		.write_to_file(out_path.clone())
		.expect("Couldn't write bindings!");
	{
		//replace 
		// ::std::os::raw::c_int  with i32
		// ::std::os::raw::c_uint with u32
		// ::std::os::raw::c_uchar with u8
		let contents = fs::read_to_string(out_path.clone()).unwrap().
			replace("::std::os::raw::c_int", "i32").
			replace("::std::os::raw::c_uint", "u32").
			replace("::std::os::raw::c_uchar", "u8");
		//replace enumerations: should be i32, compiles as u32 at some systems (as there is no fixed ANSI-C definition)
		let re_enumdef = Regex::new(
			r"(?m)^\s*pub type e([a-zA-Z_0-9]+)\s*=\s*u32;$",
		)
		.unwrap();
		let contents = re_enumdef.replace_all(&contents, "pub type e$1 = i32;");
		let mut file = OpenOptions::new().write(true).truncate(true).open(out_path.clone()).unwrap();
		file.write(contents.as_bytes()).unwrap();
	}

	//Generate enumerations
	{
		let enums = vec![
			EnumDef {
				name: "CapeRegistryValueType",
				prefix: "CapeRegVal",
				bitfield: false,
			},
			EnumDef {
				name: "CapeValueType",
				prefix: "CapeValueType",
				bitfield: false,
			},
			EnumDef {
				name: "CapePMCServiceType",
				prefix: "PMCService",
				bitfield: false,
			},
			EnumDef {
				name: "CapePMCRegistrationFlags",
				prefix: "CapePMCRegistrationFlag",
				bitfield: true,
			},
			EnumDef {
				name: "CapePMCCreationFlags",
				prefix: "CapePMCCreationFlag",
				bitfield: true,
			},
		];
		let mut enum_map: HashMap<&str, EnumInfo> = HashMap::new();
		for e in &enums {
			enum_map.insert(
				e.name,
				EnumInfo {
					comment: String::new(),
					values: Vec::new(),
				},
			);
		}
		let code_file = File::open(out_path).unwrap();
		let out_file_cape_err =
			File::create(cdir_path.join("..").join("cape_result_value.rs")).unwrap();
		let re_err = Regex::new(
			r"^\s*pub\s+const\s+COBIAERR_([A-Za-z_][A-Za-z_0-9]*)\s*:\s*u32\s*=\s*(\d+)\s*;\s*$",
		)
		.unwrap();
		let re_doc = Regex::new(r#"^#\[doc\s*=\s*\"(.*)\"\s*\]\s*$"#).unwrap();
		let re_enum_decl = Regex::new(r"^\s*pub\s+type\s+e([A-Za-z_][A-Za-z_0-9]+)").unwrap();
		let re_enum_mem= Regex::new(r"^\s*pub\s+const\s+e([A-Za-z][A-Za-z0-9]*)_([A-Za-z][A-Za-z0-9_]*)\s*:\s*e([A-Za-z][A-Za-z0-9]*)\s*=\s*((?:|0x)\d+)\s*;\s*$").unwrap();
		let mut out_file_cape_err = LineWriter::new(out_file_cape_err);
		let mut doc = String::new();
		let mut doc_read = false;
		let mut continuation_line: Option<String> = None;
		for line in io::BufReader::new(code_file).lines() {
			let mut line = line.unwrap();
			if let Some(c) = continuation_line {
				line = c + &line;
				continuation_line = None;
			}
			if line.starts_with("pub const ") && !line.ends_with(';') {
				//this is an enum member that is split into multiple lines
				continuation_line = Some(line);
				continue;
			}
			if let Some(cap) = re_err.captures(&line) {
				let name = cap[1].to_uppercase();
				out_file_cape_err
					.write_all(
						format!("pub const COBIAERR_{} : u32 = {};\n", &name, &cap[2]).as_bytes(),
					)
					.unwrap();
				doc_read = false;
				continue;
			}
			if let Some(cap) = re_doc.captures(&line) {
				doc = cap[1].to_string();
				doc_read = true;
				continue;
			}
			if doc_read {
				doc_read = false;
				if let Some(cap) = re_enum_decl.captures(&line) {
					let enum_name = cap[1].to_string();
					let enum_info = enum_map.get_mut(&enum_name[..]);
					if let Some(enum_def) = enum_info {
						enum_def.comment.clone_from(&doc);
					}
					continue;
				}
			}
			if let Some(cap) = re_enum_mem.captures(&line) {
				let mem_name = cap[1].to_string();
				let enum_name = cap[2].to_string().replace("_", "");
				let mem_name_2 = cap[3].to_string(); //this construct is here because regex panics on back references (!)
				let enum_val = cap[4].to_string();
				if mem_name.eq(&mem_name_2) && !enum_name.ends_with("COUNT") {
					//make sure it is valid
					if let Some(enum_info) = enum_map.get_mut(&mem_name[..]) {
						enum_info.values.push(EnumMember {
							name: enum_name,
							value: enum_val,
						});
					}
				}
				continue;
			}
		}
		//write all enums
		let out_file_enums = File::create(cdir_path.join("..").join("cobia_enums.rs")).unwrap();
		let mut out_file_enums = LineWriter::new(out_file_enums);
		out_file_enums
			.write_all("use bitflags::bitflags;\nuse std::fmt;\n\n".as_bytes())
			.unwrap();
		for enumeration in enums {
			let enum_name = enumeration.name;
			let enum_info = enum_map.get_mut(enum_name).unwrap();
			//strip off the prefix
			for m in enum_info.values.iter_mut() {
				if !m.name.starts_with(enumeration.prefix) {
					panic!("Invalid enum member name {} in enum {}", m.name, enum_name);
				}
				m.name = m.name[enumeration.prefix.len()..].to_string();
			}
			//bit flag?
			if enumeration.bitfield {
				out_file_enums.write_all(format!("bitflags! {{\n    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n    pub struct {} : i32 {{\n",enum_name).as_bytes()).unwrap();
				for mem in &enum_info.values {
					out_file_enums
						.write_all(
							format!("        const {} = {};\n", mem.name, mem.value).as_bytes(),
						)
						.unwrap();
				}
				out_file_enums
					.write_all("    }\n}\n\n".to_string().as_bytes())
					.unwrap();
			} else {
				//regular enum
				for s in enum_info.comment.split("\\n") {
					let mut s = s;
					if s.starts_with('*') {
						s = &s[1..];
					}
					if s.starts_with('!') {
						s = &s[1..];
					}
					if s.ends_with("*/") {
						s = &s[..s.len() - 2];
					}
					if s.ends_with('/') {
						s = &s[..s.len() - 1];
					}
					s = s.trim();
					if s.starts_with(r"\\sa") {
						continue;
					}
					out_file_enums
						.write_all(format!("///{}\n", s).as_bytes())
						.unwrap();
				}
				out_file_enums.write_all(format!("#[repr(i32)]\n#[derive(Debug, Copy, Clone, PartialEq, Eq)]\npub enum {} {{\n",enum_name).as_bytes()).unwrap();
				for mem in &enum_info.values {
					out_file_enums
						.write_all(format!("    {}={},\n", mem.name, mem.value).as_bytes())
						.unwrap();
				}
				out_file_enums.write_all(format!("}}\n\nimpl {} {{\n    ///convert from i32 to {}\n    ///\n    /// # Arguments\n    ///\n    /// * `value` - i32 value to be converted to {}\n    ///\n    /// # Examples\n    ///\n    /// ```\n    /// use cobia;\n",enum_name,enum_name,enum_name).as_bytes()).unwrap();
				for i in 0..enum_info.values.len() {
					out_file_enums.write_all(format!("    ///let v{}=cobia::{}::from({});\n    ///assert_eq!(v{}.unwrap(),cobia::{}::{});\n",i,enum_name,enum_info.values[i].value,i,enum_name,enum_info.values[i].name).as_bytes()).unwrap();
				}
				out_file_enums
					.write_all(
						format!(
							"    ///let v{}=cobia::{}::from(-1);\n    ///assert_eq!(v{},None);\n",
							enum_info.values.len(),
							enum_name,
							enum_info.values.len()
						)
						.as_bytes(),
					)
					.unwrap();
				out_file_enums.write_all(format!("    ///```\n    pub fn from(value: i32) -> Option<{}> {{\n        match value {{\n",enum_name).as_bytes()).unwrap();
				for mem in &enum_info.values {
					out_file_enums
						.write_all(
							format!(
								"            {} => Some({}::{}),\n",
								mem.value, enum_name, mem.name
							)
							.as_bytes(),
						)
						.unwrap();
				}
				out_file_enums
					.write_all("            _ => None,\n        }\n    }\n    /// Convert to string\n    pub fn as_string(&self) -> &str {\n        match self {\n".as_bytes())
					.unwrap();
				for mem in &enum_info.values {
					out_file_enums
						.write_all(
							format!(
								"			Self::{} => \"{}\",\n",
								mem.name, mem.name
							)
							.as_bytes(),
						)
						.unwrap();
				}
				let mut lower_case_name=String::new();
				lower_case_name+=&enum_name[0..1].to_lowercase();
				lower_case_name+=&enum_name[1..];
				let var_name=&lower_case_name;
				out_file_enums
					.write_all(format!(r#"        }}
    }}
    ///get an iterator
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    /// for {} in cobia::{}::iter() {{
    ///     println!("{{}}={{}}",{},{} as i32);
    /// }}
    /// ```
    pub fn iter() -> {}Iterator {{
		{}Iterator {{ current: 0 }}
	}}
}}

/// {} iterator
///
/// Iterates over all {} values
///
/// Example:
/// ```
/// use cobia;
/// for {} in cobia::{}::iter() {{
///     println!("{{}}={{}}",{},{} as i32);
/// }}
/// ```
pub struct {}Iterator {{
    current: i32,
}}
impl Iterator for {}Iterator {{
    type Item = {};
	fn next(&mut self) -> Option<Self::Item> {{
        if self.current >= {} {{
			None
		}} else {{
		    let result = {}::from(self.current);
		    self.current += 1;
		    result
        }}
	}}
}}
impl fmt::Display for {} {{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{
        write!(f,"{{}}",self.as_string())
	}}
}}

"#,var_name,enum_name,var_name,var_name,enum_name,enum_name,enum_name,enum_name,var_name,enum_name,var_name,var_name,enum_name,enum_name,enum_name,enum_info.values.len(),enum_name,enum_name).as_bytes())
					.unwrap();
			}
		}
		//find clang++


		//compile code generator program cidl2rs
		let cidl2rs_path = PathBuf::from("cidl2rs");
		let cidl2rs_intermediate_path = cidl2rs_path.join("obj");
		let cidl2rs_binary_path = cidl2rs_path.join("bin");
		//make object dir
		fs::create_dir_all(&cidl2rs_intermediate_path).unwrap();
		//compile cidl2rs.cpp
		let compile_output=std::process::Command::new(clangpp.clone())
			.arg("-c")
			.arg("-I")
			.arg(cobia_include_path.clone())
			.arg("-o")
			.arg(cidl2rs_intermediate_path.join("cidl2rs.o"))
			.arg("-std=c++20")
			.arg(cidl2rs_path.join("cidl2rs.cpp"))
			.output()
			.expect("could not spawn `clang++`");
		if !compile_output
			.status
			.success()
		{
			//print compile output 
			print!("{}",String::from_utf8_lossy(&compile_output.stdout));
			//print compile error output
			eprint!("{}",String::from_utf8_lossy(&compile_output.stderr));
			// Panic if the command was not successful.
			panic!("could not compile cidl2rs.cpp");
		}
		//if on Windows, compile resource
		let mut cidl2rs_resource_path : PathBuf = PathBuf::new();
		if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
			cidl2rs_resource_path=cidl2rs_intermediate_path.join("cidl2rs.res");
			let output=std::process::Command::new(rc.unwrap())
				.arg(cidl2rs_path.join("cidl2rs.rc"))
				.output()
				.expect("could not spawn `cidl2rs.rc`");
			if !output	
				.status
				.success()
			{
				//print compile output 
				print!("{}",String::from_utf8_lossy(&output.stdout));
				//print compile error output
				eprint!("{}",String::from_utf8_lossy(&output.stderr));
				// Panic if the command was not successful.
				panic!("could not compile cidl2rs.rc");
			}
			//move the res file to the object directory
			fs::rename(cidl2rs_path.join("cidl2rs.res"),cidl2rs_resource_path.clone()).unwrap();
		}
		//make output dir
		fs::create_dir_all(&cidl2rs_binary_path).unwrap();
		//run the linker
		let mut linker_command=std::process::Command::new(clangpp.clone());
		let cidl2rs_exe=if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
			cidl2rs_binary_path.join("cidl2rs.exe")
		} else {
			cidl2rs_binary_path.join("cidl2rs")
		};
		linker_command
			.arg("-o")
			.arg(cidl2rs_exe.clone())
			.arg(cidl2rs_intermediate_path.join("cidl2rs.o"));
		if !cidl2rs_resource_path.as_os_str().is_empty() {
			//Windows, add resource
			linker_command.arg(cidl2rs_resource_path);
		}
		let link_outout=linker_command
			.output()
			.expect("could not spawn linker");
		if !link_outout
			.status
			.success()
		{
			//print compile output 
			print!("{}",String::from_utf8_lossy(&link_outout.stdout));
			//print compile error output
			eprint!("{}",String::from_utf8_lossy(&link_outout.stderr));
			// Panic if the command was not successful.
			panic!("could not link cidl2rs");
		}
		//generate code for cape_open using cidl2rs
		let cape_open_mod=PathBuf::from("src").join("cape_open");
		fs::create_dir_all(&cape_open_mod).unwrap();
		let code_gen_cape_open=std::process::Command::new(cidl2rs_exe.clone())
			.arg("-o")
			.arg(cape_open_mod.join("mod.rs"))
			.arg("-c")
			.arg("crate")
			.arg("CAPEOPEN")
			.output()
			.expect("could not spawn `cidl2rs`");
		if !code_gen_cape_open
			.status
			.success()
		{
			//print compile output 
			print!("{}",String::from_utf8_lossy(&code_gen_cape_open.stdout));
			//print compile error output
			eprint!("{}",String::from_utf8_lossy(&code_gen_cape_open.stderr));
			// Panic if the command was not successful.
			panic!("could not generate capeopen namespace from CAPEOPEN type lib");
		}
		//generate code for cape_open_1_2 using cidl2rs
		let cape_open_mod_1_2=PathBuf::from("src").join("cape_open_1_2");
		fs::create_dir_all(&cape_open_mod_1_2).unwrap();
		let code_gen_cape_open=std::process::Command::new(cidl2rs_exe.clone())
			.arg("-o")
			.arg(cape_open_mod_1_2.join("mod.rs"))
			.arg("-c")
			.arg("crate")
			.arg("CAPEOPEN_1_2")
			.output()
			.expect("could not spawn `cidl2rs`");
		if !code_gen_cape_open
			.status
			.success()
		{
			//print compile output 
			print!("{}",String::from_utf8_lossy(&code_gen_cape_open.stdout));
			//print compile error output
			eprint!("{}",String::from_utf8_lossy(&code_gen_cape_open.stderr));
			// Panic if the command was not successful.
			panic!("could not generate capeopen_1_2 namespace from CAPEOPEN_1_2 type lib");
		}


	}
}
