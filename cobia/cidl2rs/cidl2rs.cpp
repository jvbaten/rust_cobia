// ThermoClientPMETest.cpp : Defines the entry point for the console application.
//
#include <COBIA.h>
#include <codecvt>
#include <iostream>
#include <string>
#include <vector>
#include <sstream>
#include <fstream>
#include <iomanip>
#include <unordered_set>
#include <unordered_map>

using namespace COBIA;

std::string to_snake_case(const std::string& raw) {
	bool allow_underscore=false;
	std::string name;
	for (char c:raw) {
		if (std::isupper(c)) {
			if (allow_underscore) {
				name+='_';
			}
			name+=static_cast<char>(std::tolower(c));
			allow_underscore=false;
		} else {
			name+=c;
			allow_underscore=true;
		}
	}
	return name;
}

class FromUTF8 {

	std::basic_string<COBIACHAR> converted;

	public:

#ifdef _WIN32

	FromUTF8(const char* utf8) {
		const int size=MultiByteToWideChar(CP_UTF8,0,utf8,-1,nullptr,0);
		auto t=new wchar_t[size];
		MultiByteToWideChar(CP_UTF8,0,utf8,-1,t,size);
		converted=t;
		delete []t;		
	}

#else

	FromUTF8(const char* utf8) {
		converted=utf8; //no conversion needed
	}

#endif

	[[nodiscard]] const COBIACHAR* c_str() const {
		return converted.c_str();
	}

};

class ToUTF8 {

	std::string converted;

	public:

#ifdef _WIN32

	ToUTF8(const COBIACHAR* str) {
		const int len=static_cast<int>(wcslen(str));
		const int size=WideCharToMultiByte(CP_UTF8,0,str,len,nullptr,0,nullptr,nullptr)+1;
		const auto utf8=new char[size];
		utf8[WideCharToMultiByte(CP_UTF8,0,str,len,utf8,size-1,nullptr,nullptr)]=0;
		converted=utf8;
		delete[]utf8;
	}

#else

	ToUTF8(const COBIACHAR* utf8) {
		converted=utf8; //no conversion needed
	}

#endif

	[[nodiscard]] const char* c_str() const {
		return converted.c_str();
	}

};

static std::ostream& operator<<(std::ostream& stream, const IDL::CapeUUID& uuid)
{	stream<<"CapeUUID::from_slice(&["<<std::setfill('0');
	for (int byte_index=0;byte_index<16;byte_index++) {
		if (byte_index) stream<<",";
		stream<<"0x"<<std::setw(2)<<std::hex<<static_cast<int>(uuid.data[byte_index])<<std::dec<<"u8";
	}
	stream<<"])";
    return stream;
}


class RustKeywords {

	std::unordered_set<std::string> keywords;

	RustKeywords() {
		keywords.insert("as");
		keywords.insert("break");
		keywords.insert("const");
		keywords.insert("continue");
		keywords.insert("crate");
		keywords.insert("else");
		keywords.insert("enum");
		keywords.insert("extern");
		keywords.insert("false");
		keywords.insert("fn");
		keywords.insert("for");
		keywords.insert("if");
		keywords.insert("impl");
		keywords.insert("in");
		keywords.insert("let");
		keywords.insert("loop");
		keywords.insert("match");
		keywords.insert("mod");
		keywords.insert("move");
		keywords.insert("mut");
		keywords.insert("pub");
		keywords.insert("ref");
		keywords.insert("return");
		keywords.insert("self");
		keywords.insert("Self");
		keywords.insert("static");
		keywords.insert("struct");
		keywords.insert("super");
		keywords.insert("trait");
		keywords.insert("true");
		keywords.insert("type");
		keywords.insert("unsafe");
		keywords.insert("use");
		keywords.insert("where");
		keywords.insert("while");
		keywords.insert("async");
		keywords.insert("await");
		keywords.insert("dyn");
		keywords.insert("abstract");
		keywords.insert("become");
		keywords.insert("box");
		keywords.insert("do");
		keywords.insert("final");
		keywords.insert("macro");
		keywords.insert("override");
		keywords.insert("priv");
		keywords.insert("typeof");
		keywords.insert("unsized");
		keywords.insert("virtual");
		keywords.insert("yield");
		keywords.insert("try");
	}

public:

	static RustKeywords& Instance() {
		static RustKeywords instance;
		return instance;
	}
	[[nodiscard]] bool IsKeyword(const std::string& word) const {
		return keywords.contains(word);
	}

};

class KnownCOBIANameSpaces {

	std::unordered_map<std::string,std::string> namespaces;

	KnownCOBIANameSpaces() {
		namespaces["CAPEOPEN"]="cape_open";
		namespaces["CAPEOPEN_1_2"]="cape_open_1_2";
	}

public:

	static KnownCOBIANameSpaces& Instance() {
		static KnownCOBIANameSpaces instance;
		return instance;
	}
	[[nodiscard]] bool IsKnownNameSpace(const std::string& word,std::string& converted) const {
		auto it=namespaces.find(word);
		if (it!=namespaces.end()) {
			converted=it->second;
			return true;
		}
		return false;
	}

};

struct MethodArgumentInfo {
	bool is_basic_data_type{};
	bool is_data_interface{};
	bool is_interface{};
	bool is_in={};
	bool is_out={};
	bool is_retval={};
	std::string rust_type_name;
	std::string raw_type_name;
	std::string provider_name;
	std::string to_raw_conversion;
	std::string raw_returned_value;
	std::string smart_pointer_type_name;
	std::string name;
	std::string from_raw_conversion;
	std::string init_value;
	std::string cobia_module_name;
	bool need_raw_conversion{false};
	bool need_unpack_rust_conversion{false};
	MethodArgumentInfo()=default;
	void FixNameSpace(std::string &named,const std::string &lib_name) {
		//if the type name starts with the local module name, strip it off
		size_t pos=named.find("::");
		size_t pos1=named.find('<');
		if ((pos!=std::string::npos)&&((pos1==std::string::npos)||(pos<pos1))) {
			if (named.substr(0,pos)==lib_name) {
				named=named.substr(pos+2);
				return;
			}
		}
		//convert namespace to known COBIA module namespace
		if (pos!=std::string::npos) {
			//prefix known CAPE-OPEN namespaces with COBIA module name
			std::string name_space=rust_type_name.substr(0,pos);
			std::string converted;
			if (KnownCOBIANameSpaces::Instance().IsKnownNameSpace(name_space,converted)) {
				rust_type_name=cobia_module_name+"::"+converted+rust_type_name.substr(pos);
			} else {
				//TODO import foreign types
			}
		}
	}
	void AddTemplateArgumentsToName(const std::string &arg_type_name,IDL::CapeIDLType &type,int expected_template_arg_count,IDL::CapeIDLInterface iface,IDL::PARSER::ExtendedCapeTypeResolver &resolver,const std::string &lib_name) {
		if (expected_template_arg_count==-1) {
			//resolve the type to check the argument count
			try {
				auto reference_type=resolver.Interface(ConstCapeString{FromUTF8(arg_type_name.c_str()).c_str()});
				expected_template_arg_count=reference_type.TemplateArgCount();
			} catch (std::exception& exception) {
				std::string message="unable to resolve interface '";
				message+=rust_type_name;
				message+="': ";
				message+=exception.what();
				throw std::runtime_error{message};
			}
		}
		if (expected_template_arg_count!=type.TemplateTypeCount()) {
			throw std::runtime_error("unexpected number of template arguments");
		}
		if (expected_template_arg_count>0) {
			rust_type_name+='<';
			smart_pointer_type_name+='<';
			for (int template_index=0;template_index<type.TemplateTypeCount();template_index++) {
				if (template_index>0) {
					rust_type_name+=',';
					smart_pointer_type_name+=',';
				}
				auto template_arg_type=type.TemplateType(template_index);
				switch (template_arg_type.Type()) {
				case IDL::CapeIDLDataType::IDLDataType_CapeTemplateArgument:
					//template arg name
					{
						CapeStringImpl template_name;
						iface.TemplateArg(template_arg_type.TemplateIndex(),template_name);
						rust_type_name+=ToUTF8(template_name.c_str()).c_str();
						smart_pointer_type_name+=ToUTF8(template_name.c_str()).c_str();
					}
					if (template_arg_type.TemplateTypeCount()>0) {
						//invalid
						throw std::runtime_error("template argument cannot have template arguments");
					}
					break;
				case IDL::CapeIDLDataType::IDLDataType_CapeInterface:
					//interface, which itself can have template arguments
					{
						CapeStringImpl _t_type_name;
						template_arg_type.Name(_t_type_name);
						if (_t_type_name==COBIATEXT("CapeObject")) {
							rust_type_name+=cobia_module_name+"::C::ICapeInterface";
							smart_pointer_type_name+=cobia_module_name+"::CapeObject";
							continue;
						} 
						std::string t_type_name=ToUTF8(_t_type_name.c_str()).c_str();
						//if the type name starts with the local module name, strip it off
						std::string fixed_name{t_type_name};
						FixNameSpace(fixed_name,lib_name);
						rust_type_name+=fixed_name;
						//split in name space and type name
						std::string name_space,type_name;
						size_t pos=t_type_name.find("::");
						if (pos!=std::string::npos) {
							name_space=t_type_name.substr(0,pos);
							type_name=t_type_name.substr(pos+2);
						} else {
							name_space=lib_name;
							type_name=t_type_name;
						}
						if (type_name.front()=='I') {
							type_name=type_name.substr(1);
						} else {
							type_name='T'+type_name;
						}
						if (name_space==lib_name) {
							//this module
							smart_pointer_type_name+=type_name;
						} else {
							//assume imported under its own namespace
							smart_pointer_type_name+=name_space+"::"+type_name;
							//note that types from this namespace must now be imported
							//TODO import foreign types
						}
						AddTemplateArgumentsToName(t_type_name,template_arg_type,-1,iface,resolver,lib_name);
					}
					break;
				default:
					//cannot be a template argument
					{
						std::string message="invalid template argument type '";
						CapeStringImpl _t_type_name;
						template_arg_type.Name(_t_type_name);
						message+=ToUTF8(_t_type_name.c_str()).c_str();
						message+="'";						
					}
				}
			}
			rust_type_name+='>';
			smart_pointer_type_name+='>';
		}
	}

	MethodArgumentInfo(
		IDL::CapeIDLMethodArgument arg,
		IDL::CapeIDLInterface iface,
		IDL::PARSER::ExtendedCapeTypeResolver &resolver,
		const std::string &lib_name,
		const std::string &_cobia_module_name,
		const std::string &native_module,
		const std::string &native_namespace
	) : cobia_module_name(_cobia_module_name) {
		CapeStringImpl _arg_name;
		arg.Name(_arg_name);
		std::string raw_name=ToUTF8(_arg_name.c_str()).c_str();
		name=to_snake_case(raw_name);
		if (RustKeywords::Instance().IsKeyword(name)) {
			name='_'+name;
		}
		//check attributes
		for (int attribute_index=0;attribute_index<arg.AttributeCount();attribute_index++) {
			CapeStringImpl att_name;
			arg.AttributeName(attribute_index,att_name);
			//in, out, retval
			if (att_name==COBIATEXT("in")) {
				is_in=true;
			} else if (att_name==COBIATEXT("out")) {
				is_out=true;
			} else if (att_name==COBIATEXT("retval")) {
				is_retval=true;
			} else if (att_name==COBIATEXT("orphan")) {
				//ignore, used for marshaling
			} else {
				std::string message="invalid attribute '";
				message+=ToUTF8(att_name.c_str()).c_str();
				message+='\'';
				throw std::runtime_error(message);
			}
		}
		if (!(is_in^is_out)) {
			throw std::runtime_error("argument must be [in] or [out]");
		}
		if (is_retval) {
			if (!is_out) {
				throw std::runtime_error("argument is [retval] but not [out]");
			}
		}
		CapeStringImpl _type_name;
		auto type=arg.DataType();
		type.Name(_type_name);
		rust_type_name=ToUTF8(_type_name.c_str()).c_str();
		int expected_template_arg_count=0;
		bool process_template_parameters{true};
		switch (type.Type()) {
		case IDL::CapeIDLDataType::IDLDataType_CapeEnumeration:
			is_basic_data_type=true;
			init_value="0";
			if (rust_type_name=="CapeEnumeration") {
				raw_type_name=cobia_module_name+"::C::CapeEnumeration";
				rust_type_name=cobia_module_name+"::CapeEnumeration";
			} else {
				//split in namespace and type name
				std::string name_space,type_name;
				size_t pos=rust_type_name.find("::");
				if (pos!=std::string::npos) {
					name_space=rust_type_name.substr(0,pos);
					type_name=rust_type_name.substr(pos+2);
				} else {
					name_space=lib_name;
					type_name=rust_type_name;
				}
				if (name_space==lib_name) {
					raw_type_name=native_module+"::"+name_space+'_'+type_name;
					rust_type_name=type_name;
				} else {
					//foreign reference - check known namespace
					if (std::string converted;KnownCOBIANameSpaces::Instance().IsKnownNameSpace(name_space,converted)) {
						//resolve from COBIA
						raw_type_name=cobia_module_name+"::C::"+name_space+"_"+type_name;
						rust_type_name=cobia_module_name+"::"+converted+"::"+type_name;
					} else {
						//assume imported under its own namespace
						raw_type_name=name_space+"::"+type_name;
						//note that types from this namespace must now be imported
						//TODO import foreign types
					}
				}
				from_raw_conversion="from";
				to_raw_conversion=" as "+raw_type_name;
				need_unpack_rust_conversion=true;
			}
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeBoolean:
			is_basic_data_type=true;
			raw_type_name=rust_type_name;
			init_value="false as CapeBoolean";
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeInteger:
			is_basic_data_type=true;
			raw_type_name=rust_type_name;
			init_value="0";
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeResult:
			is_basic_data_type=true;
			raw_type_name=rust_type_name;
			init_value="COBIAERR_NOERROR";
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeReal:
			is_basic_data_type=true;
			raw_type_name=rust_type_name;
			init_value="0.0";
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeUUID:
			is_basic_data_type=true;
			raw_type_name=rust_type_name;
			init_value="CapeUUID::null()";
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeInvalidType:
			throw std::runtime_error("invalid data type");
		case IDL::CapeIDLDataType::IDLDataType_CapeInterface:
			is_interface=true;
			expected_template_arg_count=-1;
			if (_type_name==COBIATEXT("CapeObject")) {
				expected_template_arg_count=0;
				raw_type_name=cobia_module_name+"::C::ICapeInterface";
				smart_pointer_type_name=cobia_module_name+"::CapeObject";
				to_raw_conversion=".as_cape_interface_pointer()";
				from_raw_conversion=(is_out)?"attach":"from_interface_pointer";
				raw_returned_value=".detach()";
			} else {
				//split in namespace and type name
				std::string name_space,type_name;
				size_t pos=rust_type_name.find("::");
				if (pos!=std::string::npos) {
					name_space=rust_type_name.substr(0,pos);
					type_name=rust_type_name.substr(pos+2);
				} else {
					name_space=lib_name;
					type_name=rust_type_name;
				}
				smart_pointer_type_name=type_name;
				if (smart_pointer_type_name.front()=='I') {
					smart_pointer_type_name=smart_pointer_type_name.substr(1);
				} else {
					smart_pointer_type_name='T'+smart_pointer_type_name;
				}
				if (name_space==lib_name) {
					//this module
					raw_type_name=native_module+"::"+name_space+"_"+type_name;
				} else {
					//foreign reference - check known namespace
					if (std::string converted;KnownCOBIANameSpaces::Instance().IsKnownNameSpace(name_space,converted)) {
						//resolve from COBIA
						raw_type_name=cobia_module_name+"::C::"+name_space+"_"+type_name;
						smart_pointer_type_name=cobia_module_name+"::"+converted+"::"+type_name;
					} else {
						//assume imported under its own namespace
						raw_type_name=name_space+"::"+type_name;
						//note that types from this namespace must now be imported
						//TODO import foreign types
						smart_pointer_type_name=native_namespace+"::"+smart_pointer_type_name;
					}
				}
				to_raw_conversion=".as_interface_pointer()";
				raw_returned_value=".detach()";
				from_raw_conversion=(is_out)?"attach":"from_interface_pointer";
			}
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeTemplateArgument:
			is_interface=true;
			raw_type_name=cobia_module_name+"::C::ICapeInterface";
			smart_pointer_type_name=rust_type_name;
			to_raw_conversion=".as_cape_interface_pointer()";
			from_raw_conversion="from_object";
			raw_returned_value=".detach() as *mut "+cobia_module_name+"::C::ICapeInterface";
			need_unpack_rust_conversion=true;
			//template arg name
			try {
				CapeStringImpl template_name;
				iface.TemplateArg(type.TemplateIndex(),template_name);
				rust_type_name=ToUTF8(template_name.c_str()).c_str();
			} catch (std::exception& exception) {
				std::string message="invalid template argument: ";
				message+=exception.what();
				throw std::runtime_error{message};
			}
			//template arguments
			expected_template_arg_count=0;
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeString:
			is_data_interface=true;
			raw_type_name=cobia_module_name+"::C::I"+rust_type_name;
			provider_name=rust_type_name+"Provider";
			rust_type_name+=(is_out)?"Out":"In";
			provider_name+=(is_out)?"Out":"In";
			to_raw_conversion=".as_cape_string_";
			to_raw_conversion+=(is_out)?"out":"in";
			to_raw_conversion+="() as *const "+raw_type_name;
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeArrayString:
			is_data_interface=true;
			raw_type_name=cobia_module_name+"::C::I"+rust_type_name;
			provider_name=rust_type_name+"Provider";
			rust_type_name+=(is_out)?"Out":"In";
			provider_name+=(is_out)?"Out":"In";
			to_raw_conversion=".as_cape_array_string_";
			to_raw_conversion+=(is_out)?"out":"in";
			to_raw_conversion+="() as *const "+raw_type_name;
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeValue:
			is_data_interface=true;
			raw_type_name=cobia_module_name+"::C::I"+rust_type_name;
			provider_name=rust_type_name+"Provider";
			rust_type_name+=(is_out)?"Out":"In";
			provider_name+=(is_out)?"Out":"In";
			to_raw_conversion=".as_cape_value_";
			to_raw_conversion+=(is_out)?"out":"in";
			to_raw_conversion+="() as *const "+raw_type_name;
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeArrayInteger:
			is_data_interface=true;
			raw_type_name=cobia_module_name+"::C::I"+rust_type_name;
			provider_name=rust_type_name+"Provider";
			rust_type_name+=(is_out)?"Out":"In";
			provider_name+=(is_out)?"Out":"In";
			to_raw_conversion=".as_cape_array_integer_";
			to_raw_conversion+=(is_out)?"out":"in";
			to_raw_conversion+="() as *const "+raw_type_name;
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeArrayBoolean:
			is_data_interface=true;
			raw_type_name=cobia_module_name+"::C::I"+rust_type_name;
			provider_name=rust_type_name+"Provider";
			rust_type_name+=(is_out)?"Out":"In";
			provider_name+=(is_out)?"Out":"In";
			to_raw_conversion=".as_cape_array_boolean_";
			to_raw_conversion+=(is_out)?"out":"in";
			to_raw_conversion+="() as *const "+raw_type_name;
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeArrayReal:
			is_data_interface=true;
			raw_type_name=cobia_module_name+"::C::I"+rust_type_name;
			provider_name=rust_type_name+"Provider";
			rust_type_name+=(is_out)?"Out":"In";
			provider_name+=(is_out)?"Out":"In";
			to_raw_conversion=".as_cape_array_real_";
			to_raw_conversion+=(is_out)?"out":"in";
			to_raw_conversion+="() as *const "+raw_type_name;
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeArrayValue:
			is_data_interface=true;
			raw_type_name=cobia_module_name+"::C::I"+rust_type_name;
			provider_name=rust_type_name+"Provider";
			rust_type_name+=(is_out)?"Out":"In";
			provider_name+=(is_out)?"Out":"In";
			to_raw_conversion=".as_cape_array_value_";
			to_raw_conversion+=(is_out)?"out":"in";
			to_raw_conversion+="() as *const "+raw_type_name;
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeArrayByte:
			is_data_interface=true;
			raw_type_name=cobia_module_name+"::C::I"+rust_type_name;
			provider_name=rust_type_name+"Provider";
			rust_type_name+=(is_out)?"Out":"In";
			provider_name+=(is_out)?"Out":"In";
			to_raw_conversion=".as_cape_array_byte_";
			to_raw_conversion+=(is_out)?"out":"in";
			to_raw_conversion+="() as *const "+raw_type_name;
			break;
		case IDL::CapeIDLDataType::IDLDataType_CapeArrayEnumeration:
			is_data_interface=true;
			provider_name=rust_type_name+"Provider";
			rust_type_name+=(is_out)?"Out":"In";
			provider_name+=(is_out)?"Out":"In";
			raw_type_name=cobia_module_name+"::C::ICapeArrayEnumeration";
			to_raw_conversion=".as_cape_array_enumeration_";
			to_raw_conversion+=(is_out)?"out":"in";
			to_raw_conversion+="() as *const "+raw_type_name;
			//template arg name
			if (type.TemplateTypeCount()!=1) {
				throw std::runtime_error("CapeArrayEnumeration must have one template argument");
			}
			process_template_parameters=false;
			{
				auto template_arg_type=type.TemplateType(0);
				if (template_arg_type.Type()!=IDL::IDLDataType_CapeEnumeration) {
					throw std::runtime_error("CapeArrayEnumeration template argument must be an enumeration");
				} 
				CapeStringImpl _t_type_name;
				template_arg_type.Name(_t_type_name);
				rust_type_name+='<';
				std::string t_type_name=ToUTF8(_t_type_name.c_str()).c_str();
				FixNameSpace(t_type_name,lib_name);
				rust_type_name+=t_type_name;
				rust_type_name+='>';
			}
			break;
		case IDL::IDLDataType_CapeWindowId:
			//CapeWindowId is always [in] and we pass by value
			raw_type_name=cobia_module_name+"::C::CapeWindowId";
			is_basic_data_type=true;
			if (is_out) {
				throw std::runtime_error("CapeWindowId must be [in]");
			}
			need_raw_conversion=true;
			break;
		}
		if (process_template_parameters) {
			AddTemplateArgumentsToName(rust_type_name,type,expected_template_arg_count,iface,resolver,lib_name);
		}
		if (raw_returned_value.empty()) {
			raw_returned_value=to_raw_conversion;
		}
		FixNameSpace(rust_type_name,lib_name);
		//check that we have set one of the flags:
		assert((static_cast<int>(is_basic_data_type)+static_cast<int>(is_data_interface)+static_cast<int>(is_interface))==1);
	}
	std::string smart_pointer_type_name_from_pointer() {
		if (from_raw_conversion.empty()) {
			return smart_pointer_type_name;
		}
		//smart_pointer_type_name have :: before template arguments
		std::string res;
		for (char c:smart_pointer_type_name) {
			if (c=='<') {
				res+="::";
			}
			res+=c;
		}
		res+="::";
		res+=from_raw_conversion;
		return res;
	}

	std::string data_interface_to_raw() {
		std::string res="(&";
		res+=name;
		res+=to_raw_conversion;
		res+=").cast_mut()";
		return res;
	}

	std::string convert_to_raw() {
		assert(need_raw_conversion);
		//currently only CapeWindowId
		return cobia_module_name+"::CapeWindowIdToRaw("+name+")";
	}

	std::string convert_from_raw() {
		assert(need_raw_conversion);
		//currently only CapeWindowId
		return cobia_module_name+"::CapeWindowIdFromRaw("+name+")";
	}

};

static std::string MakeCamelCase(const std::string& identifier) {
	std::string result;
	bool upper_case=true;
	for (char c:identifier) {
		if (c=='_') {
			upper_case=true;
		} else {
			if (upper_case) {
				result+=static_cast<char>(toupper(c));
				upper_case=false;
			} else {
				result+=static_cast<char>(tolower(c));
			}
		}
	}
	return result;
}

static std::string MakeNativeMethodName(const std::string& identifier) {
	std::string result;
	for (char c:identifier) {
		if ((c>='A')&&(c<='Z')) {
			result+='_';
			result+=static_cast<char>(tolower(c));
		} else {
			result+=c;
		}
	}
	if (result[0]!='_') {
		result='_'+result;
	}
	return "raw"+result;
}

int main(int argc,char* argv[]) {
	//command line arguments: cidl files
	// output goes to stdout
	if (argc<2) {
		std::cerr<<"Usage:cidl2rs [-o rust-mod-file] [-n native-module-for-interface] [-s native-namespace] [-c cobia-module-name] <cidl-file-or-lib-name> [<cidl-file> ...]\n";
		return 1;
	}
	try {
		capeInitialize();
		//parse command line options
		std::string output_file;
		std::string cobia_module_name;
		std::string this_module_name; //used in example code
		std::string native_module;
		std::string native_namespace;
		struct CommandLineOption {
			std::string &storage;
			const char *description;
		};
		std::unordered_map<std::string,CommandLineOption> options{
			{"-o",{output_file,"output file name"}},
			{"-c",{cobia_module_name,"COBIA module name"}},
			{"-m",{this_module_name,"module name as referred in example code"}},
			{"-n",{native_module,"native module name"}},
			{"-s",{native_namespace,"native namespace"}}
		};
		CapeArrayStringImpl files;
		CommandLineOption* current_option=nullptr;
		for (int argument_index=1;argument_index<argc;argument_index++) {
			auto it=options.find(argv[argument_index]);
			if (it!=options.end()) {
				if (current_option) {
					std::cerr<<"Error: missing argument for "<<current_option->description<<"\n";
					return 1;
				}
				current_option=&it->second;
				if (!current_option->storage.empty()) {
					std::cerr<<"Error: multiple specifications of "<<current_option->description<<'\n';
					return 1;
				}
			} else {
				if (current_option) {
					current_option->storage=argv[argument_index];
					current_option=nullptr;
				} else {
					files.emplace_back(FromUTF8(argv[argument_index]).c_str());
				}
			}
		}
		if (current_option) {
			std::cerr<<"Error: missing argument for "<<current_option->description<<"\n";
			return 1;
		}
		if (files.empty()) {
			std::cerr<<"Error: no input files\n";
			return 1;
		}
		//defaults:
		if (cobia_module_name.empty()) {
			cobia_module_name="cobia";
		}
		if (native_module.empty()) {
			native_module="C";
		}
		IDL::PARSER::CapeIDLParseResult parse_result=IDL::PARSER::CapeIDLParseResult::parse(files);
		if (parse_result.GetLibraryCount()<1) {
			std::cerr<<"No libraries found\n";
			return 1;
		}
		IDL::PARSER::ExtendedCapeTypeResolver resolver(parse_result);
		auto lib=parse_result.GetLibrary(0);
		//library name - for rust this must be the module name as well
		CapeStringImpl _lib_name;
		lib.Name(_lib_name);
		std::string lib_name=ToUTF8(_lib_name.c_str()).c_str();
		if (native_namespace.empty()) {
			native_namespace=lib_name;
		}
		if (this_module_name.empty()) {
			this_module_name=to_snake_case(lib_name);
			//replace capeopen by cape_open
			size_t index=this_module_name.find("capeopen");
			if (index!=std::string::npos) {
				this_module_name.replace(index,8,"cape_open");
			}
		}
		std::stringstream code;
		code<<"// This file was generated by cidl2rs\n";
		if (lib.InterfaceCount()>0) {
			code<<"use "<<cobia_module_name<<"::*;\n";
			if (lib.InterfaceCount()>0) {
				code<<"use "<<cobia_module_name<<"::cape_smart_pointer::CapeSmartPointer;\n";
			}			
			//any template?
			for (int interface_index=0;interface_index<lib.InterfaceCount();interface_index++) {
				auto iface=lib.Interface(interface_index);
				if (iface.TemplateArgCount()>0) {
					code<<"use std::marker::PhantomData;\n";
					break;
				}
			}
		} else {
			code<<"use "<<cobia_module_name<<"::CapeUUID;\n";
		}
		std::unordered_set<std::basic_string<CapeCharacter>> bitfields;
		if (lib.EnumCount()>0) {
			code<<"use std::fmt;\n";
			//any bit fields?
			for (int enum_index=0;enum_index<lib.EnumCount();enum_index++) {
				auto enm=lib.Enumeration(enum_index);
				if (enm.Count()<2) continue; //no bit fields
				bool is_bit_field=true;
				for (int item_index=0;item_index<enm.Count()&&is_bit_field;item_index++) {
					auto val=enm.ItemValue(item_index);
					if (val==0) {
						is_bit_field=false;
					} else {
						//see if power of two
						if ((val&(val-1))!=0) {
							is_bit_field=false;
						}
					}
				}
				if (is_bit_field) {
					CapeStringImpl name;
					enm.Name(name);
					bitfields.insert(name);
				}
			}
			if (!bitfields.empty()) {
				code<<"use bitflags::bitflags;\n";
			}
		}
		//library ID
		code<<"\n//library ID\n"
			  "pub const LIBRARY_ID:CapeUUID="<<lib.Uuid()<<";\n";
		//category IDs
		if (lib.CategoryIDCount()>0) {
			code<<"\n//Category IDs\n";
			for (int category_index=0;category_index<lib.CategoryIDCount();category_index++) {
				auto cat=lib.CategoryID(category_index);
				code<<"pub const CATEGORYID_";
				CapeStringImpl _cat_name;
				cat.Name(_cat_name);
				std::string cat_name=ToUTF8(_cat_name.c_str()).c_str();
				for (auto& c:cat_name) {
					code<<static_cast<char>(std::toupper(c));
				}
				code<<":CapeUUID="<<cat.Uuid()<<";\n";
			}
		}
		//interface IDs
		if (lib.InterfaceCount()>0) {
			code<<"\n//Interface IDs\n";
			for (int interface_index=0;interface_index<lib.InterfaceCount();interface_index++) {
				auto iface=lib.Interface(interface_index);
				code<<"pub const ";
				CapeStringImpl _iface_name;
				iface.Name(_iface_name);
				std::string iface_name=ToUTF8(_iface_name.c_str()).c_str();
				for (auto& c:iface_name) {
					code<<static_cast<char>(std::toupper(c));
				}
				code<<"_UUID:CapeUUID="<<iface.Uuid()<<";\n";
			}
		}
		//enumerations
		if (lib.EnumCount()>0) {
			code<<"\n//Enumerations\n\n";
			for (int enum_index=0;enum_index<lib.EnumCount();enum_index++) {
				auto enm=lib.Enumeration(enum_index);
				CapeStringImpl _enm_name;
				enm.Name(_enm_name);
				std::string enm_name=ToUTF8(_enm_name.c_str()).c_str();
				if (bitfields.contains(_enm_name)) {
					code<<"bitflags! {\n"
						<<"\t#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n"
						<<"\tpub struct "<<enm_name<<": u32 {\n";
					for (int item_index=0;item_index<enm.Count();item_index++) {
						CapeStringImpl _item_name;
						enm.ItemName(item_index,_item_name);
						std::string item_name=MakeCamelCase(ToUTF8(_item_name.c_str()).c_str());
						code<<"\t\t"<<item_name<<" = "<<enm.ItemValue(item_index)<<",\n";
					}
					code<<"\t}\n"
					    <<"}\n\n";
				} else {
					code<<"///"<<enm_name<<"\n"
						"///\n"
						"///"<<enm_name<<" enumeration\n"
						"///\n"
						"#[repr(i32)]\n"
						"#[derive(Debug,PartialEq,Eq,Clone,Copy)]\n"
						"pub enum "<<enm_name<<" {\n";
					for (int item_index=0;item_index<enm.Count();item_index++) {
						CapeStringImpl _item_name;
						enm.ItemName(item_index,_item_name);
						std::string item_name=MakeCamelCase(ToUTF8(_item_name.c_str()).c_str());
						code<<"\t"<<item_name<<" = "<<enm.ItemValue(item_index)<<",\n";
					}
					code<<"}\n"
					    <<"\n"
						"impl "<<enm_name<<" {\n"
						"\t/// Convert from i32 to "<<enm_name<<"\n"
						"\t///\n"
						"\t/// # Arguments\n"
						"\t///\n"
						"\t/// * `value` - i32 value to be converted to "<<enm_name<<"\n"
						"\t///\n"
						"\t/// # Examples\n"
						"\t///\n"
						"\t/// ```\n"
						"\t///use cobia::*;\n"
						"\t///use "<<this_module_name<<"::"<<enm_name<<";\n";
					for (int enum_index_1=0;enum_index_1<enm.Count();enum_index_1++) {
						CapeStringImpl _item_name;
						enm.ItemName(enum_index_1,_item_name);
						std::string item_name=MakeCamelCase(ToUTF8(_item_name.c_str()).c_str());
						code<<"\t///let v"<<enum_index_1<<"="<<enm_name<<"::from("<<enm.ItemValue(enum_index_1)<<");\n";
						code<<"\t///assert_eq!(v"<<enum_index_1<<".unwrap(),"<<enm_name<<"::"<<item_name<<");\n";
					}
					code<<"\t/// ```\n"
						"\tpub fn from(value: i32) -> Option<"<<enm_name<<"> {\n"
						"\t\tmatch value {\n";
					for (int item_index=0;item_index<enm.Count();item_index++) {
						CapeStringImpl _item_name;
						enm.ItemName(item_index,_item_name);
						std::string item_name=MakeCamelCase(ToUTF8(_item_name.c_str()).c_str());
						code<<"\t\t\t"<<enm.ItemValue(item_index)<<" => Some("<<enm_name<<"::"<<item_name<<"),\n";
					}
					code<<"\t\t\t_ => None,\n"
						"\t\t}\n"
						"\t}\n"
						"\t/// Convert to string\n"
						"\tpub fn as_string(&self) -> &str {\n"
						"\t\tmatch self {\n";
					for (int item_index=0;item_index<enm.Count();item_index++) {
						CapeStringImpl _item_name;
						enm.ItemName(item_index,_item_name);
						std::string item_name=MakeCamelCase(ToUTF8(_item_name.c_str()).c_str());
						code<<"\t\t\tSelf::"<<item_name<<" => \""<<item_name<<"\",\n";
					}
					std::string enum_var_name=enm_name;
					if (std::isupper(enum_var_name.front())) {
						enum_var_name.front()=std::tolower(enum_var_name.front());
					} else {
						enum_var_name='_'+enum_var_name;
					}
					code<<"\t\t}\n"
						"\t}\n"
						"\t///get an iterator\n"
						"\t///\n"
						"\t/// # Examples\n"
						"\t///\n"
						"\t/// ```\n"
						"\t/// use cobia::*;\n"
						"\t/// use "<<this_module_name<<"::"<<enm_name<<";\n"
						"\t/// for "<<enum_var_name<<" in "<<enm_name<<"::iter() {\n"
						"\t///     println!(\"{}={}\","<<enum_var_name<<","<<enum_var_name<<" as i32);\n"
						"\t/// }\n"
						"\t/// ```\n"
						"\tpub fn iter() -> "<<enm_name<<"Iterator {\n"
						"\t\t"<<enm_name<<"Iterator { current: 0 }\n"
						"\t}\n"
						"}\n\n"
						"/// "<<enm_name<<" iterator\n"
						"///\n"
						"/// Iterates over all "<<enm_name<<" values\n"
						"///\n"
						"/// # Examples\n"
						"///\n"
						"/// ```\n"
						"/// use cobia::*;\n"
						"/// use "<<this_module_name<<"::"<<enm_name<<";\n"
						"/// for "<<enum_var_name<<" in "<<enm_name<<"::iter() {\n"
						"///     println!(\"{}={}\","<<enum_var_name<<','<<enum_var_name<<" as i32);\n"
						"/// }\n"
						"/// ```\n"
						"pub struct "<<enm_name<<"Iterator {\n"
						"\tcurrent: i32,\n"
						"}\n"
						"impl Iterator for "<<enm_name<<"Iterator {\n"
						"\ttype Item = "<<enm_name<<";\n"
						"\tfn next(&mut self) -> Option<Self::Item> {\n"
						"\t\tif self.current>="<<enm.Count()<<" {\n"
						"\t\t\tNone\n"
						"\t\t} else {\n"
						"\t\t\tlet result="<<enm_name<<"::from(self.current);\n"
						"\t\t\tself.current+=1;\n"
						"\t\t\tresult\n"
						"\t\t}\n"
						"\t}\n"
						"}\n"
						"impl fmt::Display for "<<enm_name<<" {\n"
						"\tfn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n"
						"\t\twrite!(f,\"{}\",self.as_string())\n"
						"\t}\n"
						"}\n";
				}
			}
		}
		//interfaces
		if (lib.InterfaceCount()>0) {
			code<<"\n//Interfaces\n\n";
			for (int interface_index=0;interface_index<lib.InterfaceCount();interface_index++) {
				auto iface=lib.Interface(interface_index);
				CapeStringImpl _iface_name;
				iface.Name(_iface_name);
				std::string iface_name=ToUTF8(_iface_name.c_str()).c_str();
				//interface definition
				code<<"///"<<iface_name<<"\n"
					"///\n"
					"///"<<iface_name<<" interface\n"
					"///\n"
					"pub trait "<<iface_name;
				if (iface.TemplateArgCount()>0) {
					//template arguments
					code<<'<';
					for (int template_index=0;template_index<iface.TemplateArgCount();template_index++) {
						CapeStringImpl _arg_name;
						iface.TemplateArg(template_index,_arg_name);
						std::string arg_name=ToUTF8(_arg_name.c_str()).c_str();
						if (template_index>0) {
							code<<',';
						}
						code<<arg_name;
						code<<":CapeSmartPointer";
					}
					code<<'>';
				}
				code<<" {\n";
				std::vector<std::vector<MethodArgumentInfo>> method_args;
				method_args.resize(iface.MethodCount());
				for (int method_index=0;method_index<iface.MethodCount();method_index++) {
					auto method=iface.Method(method_index);
					CapeStringImpl _method_name;
					method.Name(_method_name);
					std::string method_name=ToUTF8(_method_name.c_str()).c_str();
					for (int attribute_index=0;attribute_index<method.AttributeCount();attribute_index++) {
						CapeStringImpl att_name;
						method.AttributeName(attribute_index,att_name);
						if (att_name==COBIATEXT("property_get")) {
							method_name="Get"+method_name;
						} else if (att_name==COBIATEXT("property_set")) {
							//ignore
							method_name="Set"+method_name;
						} else if (att_name==COBIATEXT("long_name")) {
							//use this name instead
							CapeStringImpl _long_name;
							method.AttributeValue(attribute_index,_long_name);
							method_name=ToUTF8(_long_name.c_str()).c_str();
						} else {
							//print error and exit
							std::cerr<<"Error: Method "<<method_name<<" of interface "<<iface_name<<" has invalid attribute "<<ToUTF8(att_name.c_str()).c_str()<<'\n';
							return 1;
						}
					}
					std::string method_name_snake_case=to_snake_case(method_name);
					code<<"\tfn "<<method_name_snake_case<<"(&mut self";
					if (method.ReturnType().Type()!=IDL::CapeIDLDataType::IDLDataType_CapeResult) {
						//print error and exit
						std::cerr<<"Error: Method "<<method_name<<" of interface "<<iface_name<<" does not return a CAPERESULT\n";
						return 1;
					}
					std::vector<std::string> retval_types;
					std::vector<MethodArgumentInfo>& args=method_args[method_index];
					args.resize(method.ArgumentCount());
					for (int argument_index=0;argument_index<method.ArgumentCount();argument_index++) {
						auto arg=method.Argument(argument_index);
						MethodArgumentInfo& arg_info=args[argument_index];
						try {
							arg_info=MethodArgumentInfo{arg,iface,resolver,lib_name,cobia_module_name,native_module,native_namespace};
						} catch (std::exception &exception) {
							CapeStringImpl _arg_name;
							arg.Name(_arg_name);
							std::cerr<<"Error: argument "<<ToUTF8(_arg_name.c_str()).c_str()<<" of method "<<method_name<<" of interface "<<iface_name<<": "<<exception.what()<<'\n';
							return 1;
						}
						// - all [out] interfaces go as return values
						// - data interfaces always are input.
						// - basic data types are only return value if [retval]
						if (arg_info.is_retval&&arg_info.is_basic_data_type) {
							retval_types.push_back(arg_info.rust_type_name);
							continue;
						}
						if (arg_info.is_interface) {
							if (arg_info.is_out) {
								retval_types.push_back(arg_info.smart_pointer_type_name);
							} else {
								code<<','<<arg_info.name<<':'<<arg_info.smart_pointer_type_name;
							}
						} else {
							code<<','<<arg_info.name<<':';
							if (arg_info.is_basic_data_type) {
								if (arg_info.is_out) {
									code<<"&mut ";
								}
							} else if (arg_info.is_data_interface) {
								code<<'&';
								if (arg_info.is_out) {
									code<<"mut ";
								}
							}
							code<<arg_info.rust_type_name;
						}
					}
					code<<") -> Result<";
					if (retval_types.size()==1) {
						//not as tuple
						code<<retval_types[0];
					} else {
						//empty or multiple values as tuple
						code<<'(';
						for (size_t retval_index=0;retval_index<retval_types.size();retval_index++) {
							if (retval_index) {
								code<<',';
							}
							code<<retval_types[retval_index];
						}
						code<<')';
					}
					code<<",COBIAError>;\n";
				}
				std::string templ_args_long;
				std::string templ_args_short;
				if (iface.TemplateArgCount()>0) {
					templ_args_long="<";
					templ_args_short="<";
					for (int template_index=0;template_index<iface.TemplateArgCount();template_index++) {
						CapeStringImpl _arg_name;
						iface.TemplateArg(template_index,_arg_name);
						std::string arg_name=ToUTF8(_arg_name.c_str()).c_str();
						if (template_index>0) {
							templ_args_long+=',';
							templ_args_short+=',';
						}
						templ_args_long+=arg_name;
						templ_args_short+=arg_name;
						templ_args_long+=":CapeSmartPointer";
					}
					templ_args_long+='>';
					templ_args_short+='>';
				}
				code<<"}\n\n"
					"pub trait "<<iface_name<<"Impl"<<templ_args_long<<" : "<<iface_name<<templ_args_short<<" {\n"
					"\ttype T: ICapeInterfaceImpl+"<<iface_name<<"Impl"<<templ_args_short<<";\n"
					"\n"
					"\tfn as_interface_pointer(&mut self) -> *mut "<<cobia_module_name<<"::C::ICapeInterface;\n"
					"\n"
					"\t///prepare "<<native_namespace<<'_'<<iface_name<<" interface and return as generic ICapeInterface pointer\n"
					"\tfn init_interface() -> "<<cobia_module_name<<"::C::ICapeInterface {\n"<<
					"\t\t"<<cobia_module_name<<"::C::ICapeInterface {\n"
					"\t\t\tme: std::ptr::null_mut(),\n"
					"\t\t\tvTbl: (&Self::T::VTABLE as *const "<<cobia_module_name<<"::C::"<<native_namespace<<'_'<<iface_name<<"_VTable).cast_mut()\n"
					"\t\t\t\tas *mut "<<cobia_module_name<<"::C::ICapeInterface_VTable,\n"
					"\t\t}\n"
					"\t}\n"
					"\t\n"
					"\tfn init<Timpl: "<<iface_name<<"Impl"<<templ_args_short<<"+ICapeInterfaceImpl>(u: &mut Timpl) {\n"
					"\t\tlet iface: *mut "<<cobia_module_name<<"::C::"<<native_namespace<<'_'<<iface_name<<" =\n"
					"\t\t\tu.as_interface_pointer() as *mut "<<native_module<<"::"<<native_namespace<<'_'<<iface_name<<";\n"
					"\t\tunsafe { (*iface).me = u.get_self() as *const Timpl as *mut std::ffi::c_void };\n"
					"\t\tu.add_interface(\n"
					"\t\t\tstd::ptr::addr_of!("<<native_module<<"::"<<native_namespace<<'_'<<iface_name<<"_UUID),\n"
					"\t\t\tiface as *mut "<<cobia_module_name<<"::C::ICapeInterface,\n"
					"\t\t);\n"
					"\t}\n"
					"\t\n";
				//methods
				std::vector<std::string> native_method_names;
				native_method_names.resize(iface.MethodCount());
				for (int method_index=0;method_index<iface.MethodCount();method_index++) {
					auto method=iface.Method(method_index);
					CapeStringImpl _method_name;
					method.Name(_method_name);
					std::string method_name=ToUTF8(_method_name.c_str()).c_str();
					for (int attribute_index=0;attribute_index<method.AttributeCount();attribute_index++) {
						CapeStringImpl att_name;
						method.AttributeName(attribute_index,att_name);
						if (att_name==COBIATEXT("property_get")) {
							method_name="Get"+method_name;
						} else if (att_name==COBIATEXT("property_set")) {
							//ignore
							method_name="Set"+method_name;
						} else if (att_name==COBIATEXT("long_name")) {
							//use this name instead
							CapeStringImpl _long_name;
							method.AttributeValue(attribute_index,_long_name);
							method_name=ToUTF8(_long_name.c_str()).c_str();
						} else {
							//print error and exit
							std::cerr<<"Error: Method "<<method_name<<" of interface "<<iface_name<<" has invalid attribute "<<ToUTF8(att_name.c_str()).c_str()<<'\n';
							return 1;
						}
					}
					native_method_names[method_index]=MakeNativeMethodName(method_name);
					code<<"\textern \"C\" fn "<<native_method_names[method_index]<<"(me: *mut std::ffi::c_void";
					std::vector<MethodArgumentInfo>& args=method_args[method_index];
					std::vector<std::string> non_null_arg_names;
					for (MethodArgumentInfo &arg_info:args) {
						code<<','<<arg_info.name<<':';
						if (arg_info.is_interface||arg_info.is_data_interface) {
							if (arg_info.is_interface&&arg_info.is_out) {
								//pointer to pointer
								code<<"*mut ";
							}
							code<<"*mut "<<arg_info.raw_type_name;
							if (arg_info.is_out||arg_info.is_interface) {
								non_null_arg_names.push_back(arg_info.name);
							}
						} else {
							assert(arg_info.is_basic_data_type);
							if (arg_info.is_out) {
								code<<"*mut ";
								non_null_arg_names.push_back(arg_info.name);
							}
							code<<arg_info.raw_type_name;
						}
					}
					code<<") -> "<<cobia_module_name<<"::C::CapeResult {\n";
					if (!non_null_arg_names.empty()) {
						code<<"\t\tif ";
						for (size_t arg_index=0;arg_index<non_null_arg_names.size();arg_index++) {
							if (arg_index>0) {
								code<<"||";
							}
							code<<non_null_arg_names[arg_index]<<".is_null()";
						}
						code<<" {\n"
							"\t\t\treturn COBIAERR_NULLPOINTER;\n"
						"\t\t}\n";
					}
					code<<"\t\tlet p = me as *mut Self::T;\n"
						"\t\tlet myself=unsafe { &mut *p };\n";
					std::vector<std::string> non_null;
					for (MethodArgumentInfo &arg_info:args) {
						if ((arg_info.is_data_interface&&arg_info.is_out)||arg_info.is_interface) {
							non_null.emplace_back(arg_info.name);
						} 
					}
					if (!non_null.empty()) {
						code<<"\t\tif ";
						bool first=true;
						for (const std::string& arg_name:non_null) {
							if (first) {
								first=false;
							} else {
								code<<"||";
							}
							code<<arg_name<<".is_null()";
						}
						code<<" {\n"
							"\t\t\treturn COBIAERR_NULLPOINTER;\n"
							"\t\t}\n";
					}
					std::vector<MethodArgumentInfo*> retvals;
					std::vector<MethodArgumentInfo*> outvals;
					for (MethodArgumentInfo &arg_info:args) {
						if (arg_info.is_data_interface) {
							if (arg_info.is_out) {
								//two-step conversion
								code<<"\t\tlet mut "<<arg_info.name<<"=unsafe{*((&"<<arg_info.name<<" as *const *mut "<<arg_info.raw_type_name<<") as *mut *mut "<<arg_info.raw_type_name<<")};\n"
									"\t\tlet mut "<<arg_info.name<<"=";
								for (char c:arg_info.rust_type_name) {
									if (c=='<') {
										code<<"::";
									} 
									code<<c;
								}
								code<<"::new(&mut "<<arg_info.name<<");\n";
							} else {
								code<<"\t\tlet ";
								code<<arg_info.name<<'=';
								for (char c:arg_info.rust_type_name) {
									if (c=='<') {
										code<<"::";
									} 
									code<<c;
								}
								code<<"::new(&"<<arg_info.name<<");\n";
							}
						} else if (arg_info.is_interface) {
							//by return value if output
							if (arg_info.is_out) {
								retvals.push_back(&arg_info);
							} else {
								code<<"\t\tlet "<<arg_info.name<<"=";
								if (arg_info.need_unpack_rust_conversion) {
									code<<"match "<<arg_info.smart_pointer_type_name_from_pointer()<<'('<<arg_info.name<<") {\n"
										"\t\t\tOk(_"<<arg_info.name<<") => _"<<arg_info.name<<",\n"
										"\t\t\tErr(e) => {return myself.set_last_error(e,\""<<iface_name<<"::"<<method_name<<"\");}\n"
										"\t\t};\n";
								} else {
									code<<arg_info.smart_pointer_type_name_from_pointer()<<'('<<arg_info.name<<");\n";
								}
							}
						} else if (arg_info.is_retval) {
							assert(arg_info.is_basic_data_type);
							//basic data type retval
							retvals.push_back(&arg_info);
						} else if (arg_info.is_out) {
							assert(arg_info.is_basic_data_type);
							//basic data type output
							outvals.push_back(&arg_info);
							code<<"\t\tlet mut _"<<arg_info.name<<':'<<arg_info.raw_type_name<<"="<<arg_info.init_value<<";\n";
						} else {
							assert(arg_info.is_basic_data_type);
							if (arg_info.need_unpack_rust_conversion) {
								code<<"\t\tlet "<<arg_info.name<<"=match "<<arg_info.rust_type_name<<"::"<<arg_info.from_raw_conversion<<'('<<arg_info.name<<") {\n"
									"\t\t\tSome(_"<<arg_info.name<<") => _"<<arg_info.name<<",\n"
									"\t\t\tNone => {return myself.set_last_error(COBIAError::Message(\"Invalid enumeration value\".to_string()),\""<<iface_name<<"::"<<method_name<<"\");}\n"
									"\t\t};\n";
							}
						}
					}
					std::string method_name_snake_case=to_snake_case(method_name);
					code<<"\t\tmatch myself."<<method_name_snake_case<<'(';
					bool first=true;
					for (MethodArgumentInfo& arg_info:args) {
						if ((arg_info.is_retval&&arg_info.is_basic_data_type)||(arg_info.is_out&&arg_info.is_interface)) {
							//return value
							continue;
						}
						if (first) {
							first=false;
						} else {
							code<<',';
						}
						if (arg_info.is_basic_data_type&&arg_info.is_out) {
							code<<"&mut _";
						} else if (arg_info.is_data_interface) {
							code<<'&';
							if (arg_info.is_out) {
								code<<"mut ";
							}
						} 
						if (arg_info.need_raw_conversion) {
							code<<arg_info.convert_from_raw();
						} else {
							code<<arg_info.name;
						}
					}
					code<<") {\n";
					//process return value(s)
					code<<"\t\t\tOk(";
					if (retvals.empty()) {
						code<<"_";
					} else {
						if (retvals.size()>1) code <<'(';
						for (size_t retval_index=0;retval_index<retvals.size();retval_index++) {
							if (retval_index) code<<',';
							code<<'_'<<retvals[retval_index]->name;
						}
						if (retvals.size()>1) code <<')';
					}
					code<<") => ";
					if (retvals.empty()&&outvals.empty()) {
						code<<"COBIAERR_NOERROR";
					} else {
						code<<"{\n";
						for (auto& arg_info:outvals) {
							code<<"\t\t\t\tunsafe{*"<<arg_info->name<<"=_"<<arg_info->name<<";}\n";
						}
						for (size_t retval_index=0;retval_index<retvals.size();retval_index++) {
							code<<"\t\t\t\tunsafe{*"<<retvals[retval_index]->name<<"=_"<<retvals[retval_index]->name<<retvals[retval_index]->raw_returned_value<<";}\n";
						}
						code<<"\t\t\t\tCOBIAERR_NOERROR\n"
							"\t\t\t}";
					}
					code<<",\n";
					code<<"\t\t\tErr(e) => myself.set_last_error(e,\""<<iface_name<<"::"<<method_name<<"\")\n"
						"\t\t}\n"
						"\t}\n"
						"\n";
				}
				//VTable definition
				code<<"\tconst VTABLE: "<<native_module<<"::"<<lib_name<<'_'<<iface_name<<"_VTable =\n"
					"\t\t"<<native_module<<"::"<<lib_name<<'_'<<iface_name<<"_VTable {\n"
					"\t\t\tbase: "<<cobia_module_name<<"::C::ICapeInterface_VTable {\n"
					"\t\t\t\taddReference: Some(Self::T::raw_add_reference),\n"
					"\t\t\t\trelease: Some(Self::T::raw_release),\n"
					"\t\t\t\tqueryInterface: Some(Self::T::raw_query_interface),\n"
					"\t\t\t\tgetLastError: Some(Self::T::raw_get_last_error),\n"
					"\t\t\t},\n";
				for (int method_index=0;method_index<iface.MethodCount();method_index++) {
					auto method=iface.Method(method_index);
					CapeStringImpl _method_name;
					method.Name(_method_name);
					std::string method_name=ToUTF8(_method_name.c_str()).c_str();
					for (int attribute_index=0;attribute_index<method.AttributeCount();attribute_index++) {
						CapeStringImpl att_name;
						method.AttributeName(attribute_index,att_name);
						if (att_name==COBIATEXT("property_get")) {
							method_name="get"+method_name;
						} else if (att_name==COBIATEXT("property_set")) {
							//ignore
							method_name="put"+method_name;
						} else if (att_name==COBIATEXT("long_name")) {
							//use this name instead
							CapeStringImpl _long_name;
							method.AttributeValue(attribute_index,_long_name);
							method_name=ToUTF8(_long_name.c_str()).c_str();
						} else {
							//print error and exit
							std::cerr<<"Error: Method "<<method_name<<" of interface "<<iface_name<<" has invalid attribute "<<ToUTF8(att_name.c_str()).c_str()<<'\n';
							return 1;
						}
					}
					code<<"\t\t\t"<<method_name<<": Some(Self::T::"<<native_method_names[method_index]<<"),\n";
				}
				code<<"\t\t};\n"
					  "}\n"
					  "\n";
				//smart pointer definition
				std::string smart_pointer_name;
				if (iface_name.front()=='I') {
					smart_pointer_name=iface_name.substr(1);
				} else {
					smart_pointer_name='T'+iface_name;
				}
				std::string pubcrate="pub";
				if (cobia_module_name=="crate") {
					pubcrate="pub(crate)";
				}
				std::string upper_case_iface_name=iface_name;
				for (char& c:upper_case_iface_name) {
					c=static_cast<char>(std::toupper(c));
				}
				code<<"#[cape_smart_pointer("<<upper_case_iface_name<<"_UUID)]\n"
					"pub struct "<<smart_pointer_name<<templ_args_long<<" {\n"
					"\t"<<pubcrate<<" interface: *mut "<<native_module<<"::"<<native_namespace<<"_"<<iface_name<<",\n";
				std::vector<std::string> template_phantom_names;
				template_phantom_names.reserve(iface.TemplateArgCount());
				for (int template_index=0;template_index<iface.TemplateArgCount();template_index++) {
					CapeStringImpl templ_name;
					iface.TemplateArg(template_index,templ_name);
					std::string template_name=ToUTF8(templ_name.c_str()).c_str();
					template_phantom_names.emplace_back();
					std::string &phantom_name=template_phantom_names.back();
					phantom_name="phantom_"+to_snake_case(template_name);
					code<<'\t'<<phantom_name<<" : PhantomData<"<<template_name<<">,\n";
				}
				code<<"}\n"
					"\n"
					"impl"<<templ_args_long<<' '<<smart_pointer_name<<templ_args_short<<" {\n"
					"\n";
				for (int method_index=0;method_index<iface.MethodCount();method_index++) {
					auto method=iface.Method(method_index);
					CapeStringImpl _method_name;
					method.Name(_method_name);
					std::string method_name=ToUTF8(_method_name.c_str()).c_str();
					std::string raw_method_name=method_name;
					for (int attribute_index=0;attribute_index<method.AttributeCount();attribute_index++) {
						CapeStringImpl att_name;
						method.AttributeName(attribute_index,att_name);
						if (att_name==COBIATEXT("property_get")) {
							method_name="Get"+method_name;
							raw_method_name="get"+raw_method_name;
						} else if (att_name==COBIATEXT("property_set")) {
							//ignore
							method_name="Set"+method_name;
							raw_method_name="put"+raw_method_name;
						} else if (att_name==COBIATEXT("long_name")) {
							//use this name instead
							CapeStringImpl _long_name;
							method.AttributeValue(attribute_index,_long_name);
							method_name=ToUTF8(_long_name.c_str()).c_str();
							raw_method_name=method_name;
						} else {
							//print error and exit
							std::cerr<<"Error: Method "<<method_name<<" of interface "<<iface_name<<" has invalid attribute "<<ToUTF8(att_name.c_str()).c_str()<<'\n';
							return 1;
						}
					}
					//convert method name to snake case
					std::string snake_case_method_name=to_snake_case(method_name);
					//determine return type
					std::vector<MethodArgumentInfo*> retvals;
					for (MethodArgumentInfo& arg_info:method_args[method_index]) {
						if ((arg_info.is_retval&&arg_info.is_basic_data_type)||(arg_info.is_interface&&arg_info.is_out)) {
							//retval basic data type as return value, any output interface as return value
							retvals.push_back(&arg_info);
						}
					}
					//header
					code<<"\tpub fn "<<snake_case_method_name;
					std::string arg_list;
					bool first_arg=true;
					for (MethodArgumentInfo& arg_info:method_args[method_index]) {
						if ((arg_info.is_retval&&arg_info.is_basic_data_type)||(arg_info.is_interface&&arg_info.is_out)) {
							//return value
							continue;
						}
						arg_list+=',';
						arg_list+=arg_info.name;
						arg_list+=':';
						std::string arg_type;
						if (arg_info.is_data_interface) {
							//create a template type
							if (first_arg) {
								code<<'<';
								first_arg=false;
							} else {
								code<<',';
							}
							std::string template_type_name="TypeOf";
							size_t index=0;
							if (arg_info.name[index]=='_') index++;
							template_type_name+=static_cast<char>(std::toupper(arg_info.name[index++]));
							bool nextUpper=false;
							for (;index<arg_info.name.size();index++) {
								if (arg_info.name[index]=='_') {
									nextUpper=true;
								} else {
									if (nextUpper) {
										template_type_name+=static_cast<char>(std::toupper(arg_info.name[index]));
										nextUpper=false;
									} else {
										template_type_name+=arg_info.name[index];
									}
								}
							}
							arg_type=template_type_name; //provider template
							code<<template_type_name<<':'<<arg_info.provider_name;
						} else if (arg_info.is_interface) {
							arg_type=arg_info.smart_pointer_type_name; 
						} else {
							arg_type=arg_info.rust_type_name; 
						}
						if (arg_info.is_out) {
							arg_list+="&mut ";
						} else {
							//basic data types by value, everything else by reference
							if (!arg_info.is_basic_data_type) {
								arg_list+='&';
							}
						}
						arg_list+=arg_type;
					}
					if (!first_arg) code<<'>';
					code<<"(&self"<<arg_list<<") -> Result<";
					if (retvals.size()!=1) {
						code<<'(';
					}
					for (size_t retval_index=0;retval_index<retvals.size();retval_index++) {
						if (retval_index) {
							code<<',';
						}
						if (retvals[retval_index]->is_interface) {
							code<<retvals[retval_index]->smart_pointer_type_name;
						} else {
							code<<retvals[retval_index]->rust_type_name;
						}
					}
					if (retvals.size()!=1) {
						code<<')';
					}
					code<<",COBIAError> {\n";
					for (auto* retval:retvals) {
						if (retval->is_basic_data_type) {
							code<<"\t\tlet mut "<<retval->name<<':'<<retval->raw_type_name<<'='<<retval->init_value<<";\n";
						} else {
							code<<"\t\tlet mut "<<retval->name<<": *mut "<<retval->raw_type_name<<"=std::ptr::null_mut();\n";
						}
					}
					code<<"\t\tlet result_code = unsafe {\n"
						"\t\t\t((*(*self.interface).vTbl)."<<raw_method_name<<".unwrap())((*self.interface).me";
					for (MethodArgumentInfo& arg_info:method_args[method_index]) {
						code<<',';
						if (arg_info.is_data_interface) {
							code<<arg_info.data_interface_to_raw();
						} else if (arg_info.is_interface) {
							if (arg_info.is_out) {
								code<<"&mut "<<arg_info.name<<" as *mut *mut "<<arg_info.raw_type_name;
							} else {
								code<<arg_info.name<<".as_interface_pointer() as *mut "<<arg_info.raw_type_name;
							}
						} else if (arg_info.is_basic_data_type) {
							if (arg_info.is_retval) {
								//output: local variable as mutable native pointer
								code<<"&mut "<<arg_info.name<<" as *mut "<<arg_info.raw_type_name;
							} else if (arg_info.is_out) {
								//output: as mutable native pointer
								code<<arg_info.name<<" as *mut "<<arg_info.raw_type_name;
							} else if (arg_info.need_raw_conversion) {
								code<<arg_info.convert_to_raw();
							} else {
								//input: by value
								code<<arg_info.name<<arg_info.to_raw_conversion;
							}
						} else {
							std::cerr<<"Error: Method "<<method_name<<" of interface "<<iface_name<<" has argument "<<arg_info.name<<" of unexpected type\n";
							return 1;
						}
					}
					code<<")\n\t\t};\n";
					for (size_t retval_index=0;retval_index<retvals.size();retval_index++) {
						if ((retvals[retval_index]->is_interface)&&(retvals[retval_index]->from_raw_conversion=="from_object")) {
							//QI will follow - this argument is an ICapeInterface that carries a reference count
							// bind to a CapeObject smart pointer without increasing reference count first
							code<<"\t\tlet "<<retvals[retval_index]->name<<"="<<cobia_module_name<<"::CapeObject::attach("<<retvals[retval_index]->name<<");\n";
						}
					}
					for (size_t retval_index=0;retval_index<retvals.size();retval_index++) {
						if (retvals[retval_index]->need_unpack_rust_conversion) {
							if (retvals[retval_index]->is_basic_data_type) {
								code<<"\t\tlet "<<retvals[retval_index]->name<<"=match "<<retvals[retval_index]->rust_type_name<<"::"<<retvals[retval_index]->from_raw_conversion<<'('<<retvals[retval_index]->name<<") {\n"
									"\t\t\tSome(_"<<retvals[retval_index]->name<<") => _"<<retvals[retval_index]->name<<",\n"
									"\t\t\tNone => {return Err(COBIAError::Message(\"Invalid enumeration value\".to_string()));}\n"
									"\t\t};\n";
							} else {
								code<<"\t\tlet "<<retvals[retval_index]->name<<"=match "<<retvals[retval_index]->smart_pointer_type_name_from_pointer()<<'(';
								if (retvals[retval_index]->from_raw_conversion=="from_object") {
									code<<'&';
								}
								code<<retvals[retval_index]->name<<") {\n"
									"\t\t\tOk(_"<<retvals[retval_index]->name<<") => _"<<retvals[retval_index]->name<<",\n"
									"\t\t\tErr(e) => {return Err(e);}\n"
									"\t\t};\n";
							}
						}
					}
					code<<"\t\tmatch result_code {\n"
						"\t\t\tCOBIAERR_NOERROR => {Ok(";
					if (retvals.size()!=1) {
						code<<'(';
					}
					for (size_t retval_index=0;retval_index<retvals.size();retval_index++) {
						if (retval_index) {
							code<<',';
						}
						if (retvals[retval_index]->is_basic_data_type||retvals[retval_index]->need_unpack_rust_conversion) {
							code<<retvals[retval_index]->name;
						} else {
							code<<retvals[retval_index]->smart_pointer_type_name_from_pointer()<<'('<<retvals[retval_index]->name<<')';
						}
					}
					if (retvals.size()!=1) {
						code<<')';
					}
					code<<")},\n"
						"\t\t\t_ => Err(COBIAError::from_object(result_code,self))\n"
						"\t\t}\n"
						"\t}\n"
						"\n";
				}
				code<<"}\n"
					"\n";
			}

		}
		//print output
		if (output_file.empty()) {
			std::cout<<code.str();
		} else {
			std::ofstream out(output_file);
			out<<code.str();
		}
	} catch (std::exception &ex) {
		//print problem and exit
		std::cerr<<"Error: "<<ex.what()<<'\n';
		return 1;
	}
	//all done
	return 0;
}

