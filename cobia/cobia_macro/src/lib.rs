use ::syn::parse::Parser;
use proc_macro::TokenStream;
use quote::quote;

enum CapeOpenObjectMacroArg {
	Interfaces(Vec<syn::Path>),
	CreateArguments(Vec<syn::Ident>),
	NewArguments(Vec<syn::Ident>),
}

impl syn::parse::Parse for CapeOpenObjectMacroArg {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		let name: syn::Ident = input.parse()?;
		input.parse::<syn::Token![=]>()?;
		if name=="interfaces" {
			let content;
			syn::braced!(content in input);
			let paths : syn::punctuated::Punctuated::<syn::Path, syn::Token![,]> = content.parse_terminated(syn::Path::parse)?;
			//convert paths to vector
			let mut items : Vec<syn::Path> = Vec::with_capacity(paths.len());
			for p in paths {
				items.push(p);
			}
			Ok(CapeOpenObjectMacroArg::Interfaces(items))
		} else if name=="create_arguments" {
			let content;
			syn::braced!(content in input);
			let idents : syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]> = content.parse_terminated(syn::Ident::parse)?;
			//convert idents to vector
			let mut items : Vec<syn::Ident> = Vec::with_capacity(idents.len());
			for p in idents {
				items.push(p);
			}
			Ok(CapeOpenObjectMacroArg::CreateArguments(items))
		} else if name=="new_arguments" {
			let content;
			syn::braced!(content in input);
			let idents : syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]> = content.parse_terminated(syn::Ident::parse)?;
			//convert idents to vector
			let mut items : Vec<syn::Ident> = Vec::with_capacity(idents.len());
			for p in idents {
				items.push(p);
			}
			Ok(CapeOpenObjectMacroArg::NewArguments(items))
		} else {
			Err(syn::parse::Error::new(name.span(),format!("Unknown argument: {}",name)))
		}
    }
}


/// The cape_object_implementation macro generates the necessary code to implement a COBIA object.
///
/// It provides the ICapeInterface implementation, implements error handling, reference counting and
/// QueryInterface.
///
/// This macro is used for a class implementation that is a COBIA object. The class must implement the
/// ICapeObject interface, and the macro will generate the necessary code to implement the COBIA object.
///
/// Arguments to this macro may include the following:
/// * `interfaces` - a set of CAPE-OPEN interfaces to be implemented. For each interface
///                  the raw functions are automatically implemented, and the class is 
///                  expected to implement the corresponding Impl trait. E.g. when 
///                  specifying `cape_open_1_2::ICapeIdentification` one must provide
///                  trait implementation `cape_open_1_2::ICapeIdentificationImpl`.
/// * `create_arguments` - if specified, a set of arguments used to intialize the fields
///                  of the struct. The arguments must match the name of the fields in the
///                  struct, and will have the same type in the struct. Any arguments present
///                  in the struct but not in the arguments will be initialized through
///                  std::default::Default::default().
/// * `new_arguments` - if specified, a set of arguments used to pass to a function `new` 
///                  that is implemented by the `impl` block of the struct, with the same
///                  arguments. These arguments must then be passed to the `create` or `try_create`.
///
/// Arguments are separated from their names by an equal sign, e.g.
///
/// ```text
/// interfaces={
///			cape_open_1_2::ICapeIdentification,
///			cape_open_1_2::ICapeCollection<cape_open_1_2::CapeUnitPort>,
///		}
/// ```
///
/// In addition to using this macro, a COBIA object class must also initialize its generated memeber
/// cobia_object_data in its constructor. This member can be set to Default::default()
///
/// The following traits must be implemented for the object:
/// - std::fmt::Display, which is used to get a description of the object for error handing; it is advised to include the object's type and name
/// - if the object is a primary PMC object that must be registered, also cobia::PMCRegisterationInfo
/// - any other interfaces that the object implements, specified through the attribute
///
/// # Example:
///
/// See the distillation_shortcut_unit.rs file in the examples/distillation_shortcut_unit crate for an example of how to use this macro.
/// See the salt_water_property_package.rs file in the examples/salt_water crate for an example of how to use this macro.

#[proc_macro_attribute]
pub fn cape_object_implementation(attr: TokenStream, item: TokenStream) -> TokenStream {
	//extract list of interface type identifiers
	let arguments = syn::punctuated::Punctuated::<CapeOpenObjectMacroArg, syn::Token![,]>::parse_terminated
		.parse2(attr.into())
		.expect("Incorrect format of cape_object_implementation attributes");
	//see which fields are specified
	let mut interfaces : Vec<syn::Path> = Vec::new();
	let mut creation_arguments : Option<CapeOpenObjectMacroArg> = None;
	for arg in arguments {
		match arg {
			CapeOpenObjectMacroArg::Interfaces(ifaces) => {
				interfaces.extend(ifaces)
			},
			CapeOpenObjectMacroArg::CreateArguments(_args) => {
				if creation_arguments.is_some() {
					panic!("Multiple create_arguments or new_arguments specified");
				}
				creation_arguments= Some(CapeOpenObjectMacroArg::CreateArguments(_args));
			},
			CapeOpenObjectMacroArg::NewArguments(_args) => {
				if creation_arguments.is_some() {
					panic!("Multiple create_arguments or new_arguments specified");
				}
				creation_arguments= Some(CapeOpenObjectMacroArg::NewArguments(_args));
			},
		}
	}
	if interfaces.len()==0 {
		panic!("No interfaces specified");
	}

	let mut struct_desc = syn::parse_macro_input!(item as syn::ItemStruct);
	let structname = &struct_desc.ident;
	let (impl_generics, ty_generics, where_clause) = struct_desc.generics.split_for_impl();
	let cobiainfostruct = quote::format_ident!("{}{}", structname.to_string(), "CobiaObjectData");
	//come up with a name for the pointer variable that is not likely the same as a field name
	let ptr_name= structname
		.to_string()
		.chars()
		.map(|c| if c.is_uppercase() { format!("_{}", c.to_lowercase()) } else { c.to_string() })
		.collect::<String>();
	let ptr_name= quote::format_ident!("{}_boxed_ptr", ptr_name);

	//println!("   generating {}...", cobiainfostruct);


	//add #cobiainfostruct field to struct, by name cobia_object_data

	let mut object_creation = proc_macro2::TokenStream::new();
	let mut create_arguments_def = proc_macro2::TokenStream::new();
	let mut create_arguments = proc_macro2::TokenStream::new();
	
	match struct_desc.fields {
		syn::Fields::Named(ref mut fields) => {
			match creation_arguments {
				None => {
					object_creation= quote! {
						std::default::Default::default()
					};
				}
				Some(CapeOpenObjectMacroArg::CreateArguments(ref args)) | Some(CapeOpenObjectMacroArg::NewArguments(ref args))  => {
					//make a dictionary of struct items and their types
					let struct_items : std::collections::HashMap<String, syn::Type> = fields
						.named
						.iter()
						.map(|f| {
							(f.ident.as_ref().unwrap().to_string(), f.ty.clone())
						})
						.collect();
					for id in args {
						let typ = struct_items.get(&id.to_string());
						if typ.is_none() {
							panic!("Field '{}' not found in struct", &id);
						}
						let typ = typ.unwrap();
						let arg = &id;
						create_arguments_def.extend(quote! {
							#arg : #typ,
						});
						create_arguments.extend(quote! {
							#arg,
						});
					}
					match creation_arguments {
						Some(CapeOpenObjectMacroArg::CreateArguments(_)) => {
							let create_arg_set=std::collections::HashSet::<String>::from_iter(args.iter().map(|x| x.to_string()));
							let mut create_arguments = proc_macro2::TokenStream::new();
							for f in fields.named.iter() {
								let id=&f.ident;
								if create_arg_set.contains(&f.ident.as_ref().unwrap().to_string()) {
									create_arguments.extend(quote! {
										#id,
									});
								} else {
									create_arguments.extend(quote! {
										#id : std::default::Default::default(),
									});
								}
							};
							create_arguments.extend(quote! {
								cobia_object_data : std::default::Default::default(),
							});
							object_creation.extend(quote! {
								Self {
									#create_arguments
								}
							});
						}
						Some(CapeOpenObjectMacroArg::NewArguments(_)) => {
							object_creation.extend(quote! {
								Self::new(#create_arguments)
							});
						}
						_ => {
							panic!("Internal error");
						}
					}

				}
				_ => {
					panic!("Internal error");
				}
			};
			fields.named.push(
				syn::Field::parse_named
					.parse2(quote! {
						/// Cobia object data for this object, which contains generated code to provide the native COBIA interfaces
						cobia_object_data : #cobiainfostruct  
					})
					.expect("Unable to format info struct"),
			);
		}
		syn::Fields::Unnamed(_) => {
			return TokenStream::from(
				syn::Error::new(
					syn::spanned::Spanned::span(&struct_desc),
					"unexpected tuple struct",
				)
				.to_compile_error(),
			);
		}
		syn::Fields::Unit => {
			return TokenStream::from(
				syn::Error::new(
					syn::spanned::Spanned::span(&struct_desc),
					"unexpected unit struct",
				)
				.to_compile_error(),
			);
		}
	}
	//a bit of additional code for each interface to be generated
	let mut struc_fields = proc_macro2::TokenStream::new();
	let mut struc_init_statements = proc_macro2::TokenStream::new();
	let mut struc_field_init = proc_macro2::TokenStream::new();
	let mut interface_impls = proc_macro2::TokenStream::new();
	//keep track of member variable name of cobiainfostruct
	let mut member_set : std::collections::HashSet<String> = std::collections::HashSet::new();
	member_set.insert("cape_object_data".to_string()); //add the cape_object_data field to the set
	//loop over all interfaces to be implemented
	for iface in interfaces {
		//corresponding Impl trait
		let mut impl_path = iface.clone();
		impl_path.segments.last_mut().unwrap().ident =
			quote::format_ident!("{}{}", impl_path.segments.last().unwrap().ident, "Impl");
		//identifier name - take last two path elements and seprate by underscore. Make lower case
		let mut struc_member_name: String;
		if iface.segments.len()==1 {
			struc_member_name=iface.segments.last().unwrap().ident.to_string();
		} else {
			struc_member_name=iface.segments[iface.segments.len()-2].ident.to_string();
			struc_member_name.push_str(&iface.segments.last().unwrap().ident.to_string());
		}
		//make snake case
		struc_member_name = struc_member_name
			.chars()
			.map(|c| if c.is_uppercase() { format!("_{}", c.to_lowercase()) } else { c.to_string() })
			.collect::<String>();
		//make unique
		let mut i = 1;
		while member_set.contains(&struc_member_name) {
			struc_member_name = format!("{}_{}", struc_member_name, i);
			i += 1;
		}
		member_set.insert(struc_member_name.clone());
		//make into ident
		let struc_member_name = quote::format_ident!("{}",struc_member_name);
		let comment:proc_macro2::TokenStream=format!("/// Native {} interface", iface.segments.last().unwrap().ident).parse().unwrap();
		//add field to struct
		struc_fields.extend(quote! {
			#comment
			#struc_member_name : cobia::C::ICapeInterface,
		});
		//add initialization statement
		struc_init_statements.extend(quote! {
			<Self as #impl_path>::init(u);
		});
		//structure field intialization
		struc_field_init.extend(quote! {
			#struc_member_name : <#structname as #impl_path>::init_interface(),
		});
		//interface implementation
		interface_impls.extend(quote! {
			impl #impl_generics #impl_path for #structname #ty_generics #where_clause {
				type T = #structname #ty_generics;
				fn as_interface_pointer(&mut self) -> *mut cobia::C::ICapeInterface{
					&mut self.cobia_object_data.#struc_member_name as *mut cobia::C::ICapeInterface
				}

			}
		 });
	}
	let mut fn_create_instance = proc_macro2::TokenStream::new();
	fn_create_instance.extend(quote! {
			fn create_instance(#ptr_name : *mut *mut cobia::C::ICapeInterface,#create_arguments_def) -> cobia::CapeResult {
				let obj:Self=#object_creation;
				let object_ptr=Box::into_raw(Box::new(obj)); //into_raw locks the object in memory - no need to pin
				let u : &mut Self= unsafe {&mut *object_ptr as &mut Self};
				#struc_init_statements
				unsafe {*#ptr_name=cobia::ICapeInterfaceImpl::init(u)};
				cobia::COBIAERR_NOERROR
			}
	});
	let mut cape_create_instance = proc_macro2::TokenStream::new();
	let provide_cape_create_instance=match creation_arguments {
		None => true,
		Some(CapeOpenObjectMacroArg::CreateArguments(args)) => args.len()==0,
		Some(CapeOpenObjectMacroArg::NewArguments(args)) => args.len()==0,
		_ => {
			panic!("Internal error");
		}
	};
	if provide_cape_create_instance {
		//move create instance into cobia::CapeCreateInstance trait
		cape_create_instance.extend(quote! {
			impl #impl_generics cobia::CapeCreateInstance for #structname #ty_generics #where_clause {
				#fn_create_instance
			}
		});
		//and don't but it in the impl block for Self
		fn_create_instance = proc_macro2::TokenStream::new();
	}

	quote! {

		#struct_desc

		/// Data structure that contains generated code to provide the native COBIA interfaces
		///
		/// This object is generated by the `cape_object_implementation` macro.

		struct #cobiainfostruct {
			/// CapeObjectData for this object
			cape_object_data : cobia::CapeObjectData,
			#struc_fields
		}

		#cape_create_instance

		use cobia::prelude::CapeSmartPointer;

		impl #impl_generics #structname #ty_generics #where_clause {
			#fn_create_instance
			/// Create a new instance of the object and return a smart pointer to it
			///
			/// This function panics if the created object does not implement the
			/// interface corresponding to the smart pointer to be returned. For a 
			/// non-panic version, use the try_create() function.
			pub(crate) fn create<T:cobia::prelude::CapeSmartPointer>(#create_arguments_def) -> T {
				let mut #ptr_name: *mut C::ICapeInterface=std::ptr::null_mut();
				Self::create_instance(&mut #ptr_name as *mut *mut C::ICapeInterface,#create_arguments);
				match T::from_cape_interface_pointer(#ptr_name) {
					Ok(smart_pointer) => smart_pointer,
					Err(e) => {
						//add and remove reference to avoid memory leak
						let _obj=CapeObject::from_interface_pointer(#ptr_name);
						panic!("Error creating smart pointer: {}", e);
					},
				}
			}
			/// Create a new instance of the object and return a smart pointer to it
			///
			/// This function returns an error if the created object does not implement the
			/// interface corresponding to the smart pointer to be returned. 
			pub(crate) fn try_create<T:cobia::prelude::CapeSmartPointer>(#create_arguments_def) -> Result<T,COBIAError> {
				let mut #ptr_name: *mut C::ICapeInterface=std::ptr::null_mut();
				Self::create_instance(&mut #ptr_name as *mut *mut C::ICapeInterface,#create_arguments);
				let p=CapeObject::from_interface_pointer(#ptr_name);
				T::from_object(&p)
			}
			/// Get a reference to the object
			/// 
			/// Note that as the reference is created from a smart pointer, the reference
			/// is valid only as long as the smart pointer is valid. The reference
			/// is not checked for multiple borrows
			///
			/// # Safety
			///
			/// The user must ensure that the smart pointer contains a valid 
			/// object, and that the object is of the type that is being borrowed. It is 
			/// not possible for the compiler to check this, as the borrow is done
			/// through the native object pointer in the interface.
			pub(crate) unsafe fn borrow<T:cobia::prelude::CapeSmartPointer>(smart_pointer:&T) -> &Self {
				let p=smart_pointer.as_cape_interface_pointer();
				let me=unsafe {(*p).me};
				let p = me as *const Self;
				unsafe {&*p}				
			}
			/// Get a mutable reference to the object
			/// 
			/// Note that as the reference is created from a smart pointer, the reference
			/// is valid only as long as the smart pointer is valid. The reference
			/// is not checked for multiple borrows
			///
			/// # Safety
			///
			/// This function is unsafe because it allows mutable access to the object. 
			/// Also the user must ensure that the smart pointer contains a valid 
			/// object, and that the object is of the type that is being borrowed. It is 
			/// not possible for the compiler to check this, as the borrow is done
			/// through the native object pointer in the interface.
			pub(crate) unsafe fn borrow_mut<T:cobia::prelude::CapeSmartPointer>(smart_pointer:&mut T) -> &mut Self {
				let p=smart_pointer.as_cape_interface_pointer();
				let me=unsafe {(*p).me};
				let p = me as *mut Self;
				unsafe {&mut *p}				
			}
		}

		impl Default for #cobiainfostruct {
			   fn default() -> #cobiainfostruct {
				#cobiainfostruct::new()
			}
		}

		impl #cobiainfostruct {
			fn new() ->  #cobiainfostruct {
				#cobiainfostruct {
					cape_object_data  : <#structname as cobia::ICapeInterfaceImpl>::create_object_data::<#structname>(),
					#struc_field_init
				}
			}
		}

		impl #impl_generics cobia::ICapeInterfaceImpl for #structname #ty_generics #where_clause {
			type T = #structname #ty_generics;
			fn get_object_data(&mut self) -> &mut cobia::CapeObjectData {
				&mut self.cobia_object_data.cape_object_data
			}
			fn get_self(&mut self) -> *mut #structname {
				self as *mut #structname
			}
		}

		#interface_impls

	}
	.into()
}


/// The cape_smart_pointer macro generates the necessary code to implement a COBIA smart pointer.
///
/// It exercises the ICapeInterface implementation, provides error handling, handles reference counting 
/// and handles QueryInterface.
///
/// It is expected that the interface data member is named 'interface'. Also the wrapper provides
///
/// fn from_interface_pointer(interface: *mut Interface) ->  Self
///
/// where Interface is the interface type that the wrapper implements; from_interface_pointer should
/// increase the reference count of the object. A second construction is needed that does not 
/// increase the reference count, to construct the smart pointer from a return value that already
/// has its reference count adjusted (e.g from query_interface or any function that returns an 
/// interface pointer):
///
/// fn attach(interface: *mut Interface) ->  Self
///
/// The contained interface pointer is released upon Drop, except when the smart pointer is 
/// detached. The smart pointer can be cloned, which increases the reference count of the
/// contained interface pointer.
///
/// This macro is used for a wrapper class implementation that refer to a COBIA object. This
/// macro will generate the necessary code to implement the CapeSmartPointer, Drop (=release) 
/// and Clone (=addReference) traits.
///
/// Code that uses the cape_smart_pointer macro is typically provided (CobiaCollectionBase, 
/// CobiaIdentification, ...) or generated from type information (see e.g. the cape_open_1_2 module).
///
/// The attribute passed to the interface is the interface ID. The interface type is derived
/// from the 'interface' member, which must be a *mut of the interface type.
/// 

#[proc_macro_attribute]
pub fn cape_smart_pointer(attr: TokenStream, item: TokenStream) -> TokenStream {
	//parse input
	let struct_desc = syn::parse_macro_input!(item as syn::ItemStruct);
	let structname = &struct_desc.ident;
	//println!("   generating smart pointer members for {}...", structname);
	//determine interface type
	let mut interface_type = None;
	match struct_desc.fields {
		syn::Fields::Named(ref fields) => {
			//find the interface field
			for field in fields.named.iter() {
				if field.ident.as_ref().expect("Unable to obtain field identifier") == "interface" {
					match field.ty {
						syn::Type::Ptr(ref type_ptr) => {
							match type_ptr.mutability {
								Some(_) => {}
								None => {
									return TokenStream::from(
										syn::Error::new(
											syn::spanned::Spanned::span(&field),
											"invalid interface member; must be a *mut to an interface type",
										)
										.to_compile_error(),
									);
								}
							}
							match *type_ptr.elem {
								syn::Type::Path(ref type_path) => {
									interface_type = Some(type_path.path.clone());
								}
								_ => {
									return TokenStream::from(
										syn::Error::new(
											syn::spanned::Spanned::span(&field),
											"invalid interface member; must be a *mut to an interface type",
										)
										.to_compile_error(),
									);
								}
							}
						}
						_ => {
							return TokenStream::from(
								syn::Error::new(
									syn::spanned::Spanned::span(&field),
									"invalid interface member; must be a *mut to an interface type",
								)
								.to_compile_error(),
							);
						}

					}
					break;
				}
			}
		}
		syn::Fields::Unnamed(_) => {
			return TokenStream::from(
				syn::Error::new(
					syn::spanned::Spanned::span(&struct_desc),
					"unexpected tuple struct",
				)
				.to_compile_error(),
			);
		}
		syn::Fields::Unit => {
			return TokenStream::from(
				syn::Error::new(
					syn::spanned::Spanned::span(&struct_desc),
					"unexpected unit struct",
				)
				.to_compile_error(),
			);
		}
	}
	if interface_type.is_none() {
		return TokenStream::from(
			syn::Error::new(
				syn::spanned::Spanned::span(&struct_desc),
				"interface field not found",
			)
			.to_compile_error(),
		);
	}
	let interface_type=interface_type.expect("Invalid interface type");
	//a bit of additional code for each interface to be generated
	let interface_id=proc_macro2::TokenStream::from(attr);
	let genericdef = struct_desc.generics.clone();
	let mut genericref = syn::Generics{
			lt_token: None,
			params: syn::punctuated::Punctuated::new(),
			gt_token: None,
			where_clause: None,
		};
	if !genericdef.params.is_empty() {
		genericref=genericdef.clone();
		genericref.params.iter_mut().for_each(|param| {
			match param {
				syn::GenericParam::Type(type_param) => {
					type_param.attrs.clear();
					type_param.colon_token=None;
					type_param.bounds=syn::punctuated::Punctuated::new();
					type_param.eq_token=None;
					type_param.default=None;
				},
				_ => {}
			}
		});
	}
	

	//additional struct initializers
	let mut extra_initializers = proc_macro2::TokenStream::new();
	for field in struct_desc.fields.iter() {
		let field = field.ident.as_ref().expect("Unable to obtain field identifier");
		if  field == "interface" {
			continue;
		}
		extra_initializers.extend(quote! {
			#field : std::default::Default::default(),
		});
	}

	quote! {

		#struct_desc

		impl #genericdef CapeSmartPointer for #structname #genericref {
			type Interface = #interface_type;
			fn as_interface_pointer(&self) -> *mut Self::Interface {
				self.interface
			}
			fn as_cape_interface_pointer(&self) -> *mut C::ICapeInterface {
				self.interface as *mut C::ICapeInterface
			}
			fn get_interface_id() -> &'static CapeUUID {
				& #interface_id
			}
			fn from_object<T:CapeSmartPointer>(smart_pointer : &T) -> Result<Self,COBIAError> {
				let mut interface : * mut C::ICapeInterface=std::ptr::null_mut();
				let res=unsafe {
					let other_interface=smart_pointer.as_cape_interface_pointer();
					((*(*other_interface).vTbl).queryInterface.unwrap())((*other_interface).me,Self::get_interface_id(),&mut interface as *mut *mut C::ICapeInterface)
				};
				if res==COBIAERR_NOERROR {
					if interface.is_null() {
						Err(COBIAError::Code(COBIAERR_NULLPOINTER))
					} else {
						Ok(Self::attach(interface as *mut #interface_type))
					}
				} else {
					Err(COBIAError::from_object(res,smart_pointer))
				}
			}
			fn from_interface_pointer(interface: *mut Self::Interface) ->  Self {
				if interface.is_null() {
					panic!("Null pointer in creation of ICapeIdentificationSmartPtr");
				}
				unsafe {((*(*(interface as *mut C::ICapeInterface)).vTbl).addReference.unwrap())((*interface).me)};
				Self {
					interface,
					#extra_initializers
				}
			}
			fn attach(interface: *mut Self::Interface) ->  Self {
				if interface.is_null() {
					panic!("Null pointer in creation of ICapeIdentificationSmartPtr");
				}
				Self {
					interface,
					#extra_initializers
				}
			}
			fn detach(self) ->  *mut Self::Interface {
				let res=self.interface;
				std::mem::forget(self); //don't call drop
				res
			}
			fn from_cape_interface_pointer(interface : *mut C::ICapeInterface) -> Result<Self,COBIAError>  {
				let mut my_interface : * mut C::ICapeInterface=std::ptr::null_mut();
				let res=unsafe {
					((*(*interface).vTbl).queryInterface.unwrap())((*interface).me,Self::get_interface_id(),&mut my_interface as *mut *mut C::ICapeInterface)
				};
				if res==COBIAERR_NOERROR {
					if my_interface.is_null() {
						Err(COBIAError::Code(COBIAERR_NULLPOINTER))
					} else {
						Ok(Self::attach(my_interface as *mut #interface_type))
					}
				} else {
					Err(COBIAError::from_cape_interface_pointer(res,interface))
				}
			}
			fn last_error(&self) -> Option<CapeError> {
				let mut interface : * mut C::ICapeError=std::ptr::null_mut();
				let res=unsafe {
					((*(*(self.interface as *mut C::ICapeInterface)).vTbl).getLastError.unwrap())((*self.interface).me,&mut interface as *mut *mut C::ICapeError)
				};
				if res==COBIAERR_NOERROR {
					if interface.is_null() {
						None
					} else {
						Some(CapeError::attach(interface))
					}
				} else {
					None
				}
			}
		}

		impl #genericdef Drop for #structname #genericref {
			fn drop(&mut self) {
				let interface=self.as_cape_interface_pointer();
				unsafe {
					((*(*interface).vTbl).release.unwrap())((*interface).me);
				}
			}
		}

		impl #genericdef Clone for #structname #genericref {
			fn clone(&self) -> Self {
				let interface=self.as_cape_interface_pointer();
				Self::from_interface_pointer(interface as *mut #interface_type)
			}
		}

	}
	.into()
}

