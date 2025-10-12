
/// Unconstrained mole fraction derivatives for any property
///
/// The water mole fraction is ignored in the calculation; the salinity is determined
/// from the NaCl mole fraction only (assuming that the mole fractions are normalized).
/// So the derivative w.r.t. water mole fraction is identically zero.
///
/// # Arguments
/// * `d_x_nacl` - NaCl mole fraction derivative, or an error
///
/// # Returns
/// The mole fraction derivatives with respect to both compounds, or an error.
pub fn unconstrained_dx(d_x_nacl:Result<f64,String>) -> Result<[f64;2],String> {
	match d_x_nacl {
		Ok(d_x_nacl) => {
			Ok([0.0,d_x_nacl]) //x[water] is unused
		},
		Err(e) => Err(e)
	}		
}

///Mole number derivatives for intensive property for a total number of moles of 1
///
/// Mole numbers derivatives for intensive properties in CAPE-OPEN are defined as 
/// the derivatives of the property w.r.t. the mole numbers, for a total number of
/// moles of 1.
///
/// # Arguments:
/// * `d_x_nacl` - NaCl mole fraction derivative, or an error
///
/// # Returns
/// The mole number derivatives with respect to both compounds, or an error.
pub fn intenstive_dn(d_x_nacl:Result<f64,String>,x_nacl: f64) -> Result<[f64;2],String> {
	match d_x_nacl {
		Ok(d_x_nacl) => {
			Ok([-x_nacl*d_x_nacl,d_x_nacl*(1.0-x_nacl)])
		},
		Err(e) => Err(e)
	}		
}

///Mole number derivatives for extenstive property for a total number of moles of 1
///
/// Mole numbers derivatives for extensive properties in CAPE-OPEN are defined as 
/// the derivatives of the total property (the molar property multiplied by the number
/// of moles) w.r.t. the mole numbers, for a total number of moles of 1.
///
/// # Arguments:
/// * `d_x_nacl` - NaCl mole fraction derivative, or an error
/// * `prop_value` - density, mol/m3
/// * `x_nacl` - the mole fraction of NaCl
///
/// # Returns
/// The mole number derivatives with respect to both compounds, or an error.
pub fn extenstive_dn(d_x_nacl:Result<f64,String>,prop_value:Result<f64,String>,x_nacl: f64) -> Result<[f64;2],String> {
	match d_x_nacl {
		Ok(d_x_nacl) => {
			match prop_value {
				Ok(prop_value) => Ok([prop_value-x_nacl*d_x_nacl,prop_value-d_x_nacl*(x_nacl-1.0)]),
				Err(e) => Err(e)
			}
		},
		Err(e) => Err(e)
	}		
}

///Mole number derivatives for molar volume for a total number of moles of 1
///
/// Mole numbers derivatives for extensive properties in CAPE-OPEN are defined as 
/// the derivatives of the total property (the molar property multiplied by the number
/// of moles) w.r.t. the mole numbers, for a total number of moles of 1.
///
/// This routine gives the mole number derivatives of volume, given density.
///
/// # Arguments:
/// * `d_x_nacl` - NaCl mole fraction derivative of density, or an error
/// * `prop_value` - density, mol/m3
/// * `x_nacl` - the mole fraction of NaCl
///
/// # Returns
/// The mole number derivatives with respect to both compounds, or an error.
pub fn extenstive_reciprocal_dn(d_x_nacl:Result<f64,String>,prop_value:Result<f64,String>,x_nacl: f64) -> Result<[f64;2],String> {
	match d_x_nacl {
		Ok(d_x_nacl) => {
			match prop_value {
				Ok(prop_value) => {
					let denom=1.0/(prop_value*prop_value);
					Ok([(x_nacl*d_x_nacl+prop_value)*denom,(prop_value+ d_x_nacl*(x_nacl-1.0))*denom])
				},
				Err(e) => Err(e)
			}
		},
		Err(e) => Err(e)
	}		
}

/// Viscosity calculation
/// 
/// Viscosity, Pa*s, is calculated from:
///
/// Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
/// 
/// It is pressure indepedendent, valid in the range of \[273.15-453.15\] K and x\[NaCl\]=\[0,0.05159128949\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Viscosity, Pa*s, or an error
pub fn viscosity(temperature: f64,x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for viscosity correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05159128949 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05159128949] mol/mol range for viscosity correlation",x_nacl));
	}
	let term1=temperature*temperature;
	let term2=x_nacl*x_nacl;
	let term3=0.4042772e2*x_nacl+0.1801528e2;
	let term4=term3*term3;
	Ok(0.9340370609e-5/(0.157e0*term1-0.65361298e2*temperature+0.6711409854e4)/term4*(term1*(x_nacl-0.7218318162e-1)*x_nacl+temperature*(-0.6982851317e3*term2+0.5458304152e2*x_nacl)+0.1395387535e6*term2-0.7306275892e4*x_nacl+0.2337263018e3)*(term1-0.4163140000e3*temperature+0.1914133818e6))
}

/// Calculation of viscosity derivative w.r.t. pressure
/// 
/// Viscosity, Pa*s, is calculated from:
///
/// Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
/// 
/// It is pressure indepedendent, valid in the range of \[273.15-453.15\] K and x\[NaCl\]=\[0,0.05159128949\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Viscosity derivative w.r.t. pressure, Pa*s/Pa, or an error
pub fn viscosity_d_pressure(temperature: f64,x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for viscosity correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05159128949 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05159128949] mol/mol range for viscosity correlation",x_nacl));
	}
	Ok(0.0)
}

/// Calculation of viscosity derivative w.r.t. temperature
/// 
/// Viscosity, Pa*s, is calculated from:
///
/// Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
/// 
/// It is pressure indepedendent, valid in the range of \[273.15-453.15\] K and x\[NaCl\]=\[0,0.05159128949\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Viscosity derivative w.r.t. temperature, Pa*s/K, or an error
pub fn viscosity_d_temperature(temperature: f64,x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for viscosity correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05159128949 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05159128949] mol/mol range for viscosity correlation",x_nacl));
	}
	let term1=x_nacl*x_nacl;
	let term2=temperature*temperature;
	let term3=term2*term2;
	let term4=0.4042772e2*x_nacl+0.1801528e2;
	let term5=term4*term4;
	let term6=0.157e0*term2-0.65361298e2*temperature+0.6711409854e4;
	let term7=term6*term6;
	Ok(1.0/term7/term5*(term3*temperature*(0.2932876371e-5*term1-0.2117043476e-6*x_nacl)+term3*(-0.3465986968e-2*term1+0.256313624e-3*x_nacl)+term2*temperature*(0.1611670969e1*term1-0.1214375971e0*x_nacl)+term2*(-0.307940248e3*term1+0.2290305268e2*x_nacl+0.1e-9)+temperature*(-0.3960896e3*term1-0.139556869e4*x_nacl-0.1019087985e3)+0.2121302976e5-0.81689311e4*x_nacl+0.428573361e7*term1))
}

/// Calculation of viscosity derivative w.r.t. mole fraction of NaCl
/// 
/// Viscosity, Pa*s, is calculated from:
///
/// Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
/// 
/// It is pressure indepedendent, valid in the range of \[273.15-453.15\] K and x\[NaCl\]=\[0,0.05159128949\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Viscosity derivative w.r.t. mole fraction of NaCl, Pa*s, or an error
pub fn viscosity_d_x_nacl(temperature: f64,x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for viscosity correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05159128949 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05159128949] mol/mol range for viscosity correlation",x_nacl));
	}
	let term1=temperature*temperature;
	let term2=x_nacl*x_nacl;
	let term3=0.4042772e2*x_nacl+0.1801528e2;
	let term4=term3*term3;
	Ok(1.0/term4/term3/(0.157e0*term1-0.65361298e2*temperature+0.6711409854e4)*(term1-0.416314e3*temperature+0.1914133818e6)*(-0.1405937326e1+0.3637958667e-3*term1*x_nacl-0.1214622007e-4*term1+0.2e-9*term2*temperature-0.255611125e0*temperature*x_nacl+0.91846552e-2*temperature+0.4971912439e2*x_nacl))
}

/// Thermal conductivity
/// 
/// Thermal conductivity, W/m/K, is calculated from:
///
/// Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
/// 
/// It is pressure indepedendent, valid in the range of \[273.15-453.15\] K and x\[NaCl\]=\[0,0.05545875778\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Thermal conductivity, W/m/K, or an error
pub fn thermal_conductivity(temperature: f64,x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for thermal conductivity correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05545875778 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05545875778] mol/mol range for thermal conductivity correlation",x_nacl));
	}
	let term1=f64::powf(((0.3488753105e9-0.5053465000e6*temperature)*x_nacl+0.145698577e9-0.225191e6*temperature)/(0.3488753105e9*x_nacl+0.145698577e9),0.001);
	let term2=term1*term1;
	let term3=term2*term2;
	let term4=term3*term3;
	let term5=term4*term4;
	let term6=term5*term5;
	let term7=term6*term6;
	let term8=term7*term7;
	let term9=term8*term8;
	let term10=0.4042772000e2*x_nacl+0.1801528000e2;
	let term11=1.0/term10;
	let term12=f64::ln(term11*(0.9714341400e4*x_nacl+0.4323667200e4));
	let term13=f64::exp(0.1e1/temperature*term11*(term9*term7*term4*term3*term1*((0.9292070654e2*x_nacl+0.4140704809e2)*temperature-0.1603843028e5*x_nacl-0.6184052623e4)+term12*temperature*term10));
	Ok(0.001*term13)
}

/// Calculation of thermal conductivity derivative w.r.t. temperature
/// 
/// Thermal conductivity, W/m/K, is calculated from:
///
/// Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
/// 
/// It is pressure indepedendent, valid in the range of \[273.15-453.15\] K and x\[NaCl\]=\[0,0.05545875778\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of thermal conductivity w.r.t. temperature, W/m/K^2, or an error
pub fn thermal_conductivity_d_temperature(temperature: f64,x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for thermal conductivity correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05545875778 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05545875778] mol/mol range for thermal conductivity correlation",x_nacl));
	}
	let term1=0.9292070654e2*x_nacl+0.4140704809e2;
	let term2=1.0/(0.3488753105e9*x_nacl+0.145698577e9);
	let term3=f64::powf(term2*((0.3488753105e9-0.5053465000e6*temperature)*x_nacl+0.145698577e9-0.225191e6*temperature),0.001);
	let term4=term3*term3;
	let term5=term4*term4;
	let term6=term5*term5;
	let term7=term6*term6;
	let term8=term7*term7;
	let term9=term8*term8;
	let term10=term9*term9;
	let term11=term10*term10;
	let term12=term11*term9*term6*term5*term3;
	let term13=temperature*term1-0.1603843028e5*x_nacl-0.6184052623e4;
	let term14=term11*term11;
	let term15=0.4042772000e2*x_nacl+0.1801528000e2;
	let term16=1.0/term15;
	let term17=f64::ln(term16*(0.9714341400e4*x_nacl+0.4323667200e4));
	let term18=1.0/temperature;
	let term19=term16*(term17*temperature*term15+term12*term13);
	let term20=temperature*temperature;
	let term21=f64::exp(term18*term19);
	Ok(0.001*term21*(term18*term16*(term12*term1+0.333e3/0.1000e4*term2*(-0.5053465000e6*x_nacl-0.225191e6)/term14/term10/term7/term6/term4/term3*term13+term17*term15)-0.1e1/term20*term19))
}

/// Calculation of viscosity derivative w.r.t. pressure
/// 
/// Thermal conductivity, W/m/K, is calculated from:
///
/// Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
/// 
/// It is pressure indepedendent, valid in the range of \[273.15-453.15\] K and x\[NaCl\]=\[0,0.05545875778\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of thermal conductivity w.r.t. pressure, W/m/K/Pa, or an error
pub fn thermal_conductivity_d_pressure(temperature: f64,x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for thermal conductivity correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05545875778 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05545875778] mol/mol range for thermal conductivity correlation",x_nacl));
	}
	Ok(0.0)
}

/// Calculation of viscosity derivative w.r.t. NaCl mole fraction
/// 
/// Thermal conductivity, W/m/K, is calculated from:
///
/// Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
/// 
/// It is pressure indepedendent, valid in the range of \[273.15-453.15\] K and x\[NaCl\]=\[0,0.05545875778\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of thermal conductivity w.r.t. NaCl mole fraction, W/m/K, or an error
pub fn thermal_conductivity_d_x_nacl(temperature: f64,x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for thermal conductivity correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05545875778 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05545875778] mol/mol range for thermal conductivity correlation",x_nacl));
	}
	let term1=0.3488753105e9-0.5053465000e6*temperature;
	let term2=x_nacl*term1+0.145698577e9-0.225191e6*temperature;
	let term3=0.3488753105e9*x_nacl+0.145698577e9;
	let term4=1.0/term3;
	let term5=f64::powf(term4*term2,0.001);
	let term6=term5*term5;
	let term7=term6*term6;
	let term8=term7*term7;
	let term9=term8*term8;
	let term10=term9*term9;
	let term11=term10*term10;
	let term12=term11*term11;
	let term13=term12*term12;
	let term14=term13*term11*term8*term7*term5;
	let term15=(0.9292070654e2*x_nacl+0.4140704809e2)*temperature-0.1603843028e5*x_nacl-0.6184052623e4;
	let term16=term13*term13;
	let term17=term3*term3;
	let term18=0.9714341400e4*x_nacl+0.4323667200e4;
	let term19=0.4042772000e2*x_nacl+0.1801528000e2;
	let term20=1.0/term19;
	let term21=f64::ln(term20*term18);
	let term22=term19*term19;
	let term23=1.0/term22;
	let term24=1.0/temperature;
	let term25=term21*temperature*term19+term14*term15;
	let term26=f64::exp(term24*term20*term25);
	Ok(0.001*term26*(term24*term20*(term14*(0.9292070654e2*temperature-0.1603843028e5)+0.333e3/0.1000e4*(term4*term1-0.3488753105e9/term17*term2)/term16/term12/term9/term8/term6/term5*term15+0.4042772000e2*temperature*term21+0.1e1/term18*(0.9714341400e4*term20-0.4042772000e2*term23*term18)*temperature*term22)-0.4042772000e2*term24*term23*term25))
}

/// Enthalpy
/// 
/// Enthalpy, J/mol, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-393.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.04033898281\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Enthalpy, J/mol, or an error
pub fn enthalpy(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>393.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,393.15] K range for enthalpy correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.04033898281 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.04033898281] mol/mol range for enthalpy correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for enthalpy correlation",pressure));
	}
	let term1=temperature-0.27315e3;
	let term2=term1*term1;
	let term3=term2*term1;
	let term4=0.4042772e2*x_nacl+0.1801528e2;
	let term5=1.0/term4*x_nacl;
	let term6=x_nacl*x_nacl;
	let term7=term4*term4;
	let term8=1.0/term7*term6;
	Ok((0.4042772e-1*x_nacl+0.1801528e-1)*(-0.1147654065e7+0.420207e4*temperature-0.535e0*term2+0.4e-2*term3-0.58443e2*(-0.2161173520e7+0.1842024007e8*term5+0.9572823819e10*term8-0.2886581450e13/term7/term4*term6*x_nacl+0.782607e4*temperature-0.441733e2*term2+0.21394e0*term3-0.1163646884e7*term1*term5+0.9490064212e8*term1*term8+0.5685340884e4*term2*term5)*term5+0.1e-5*(0.1881946590e4-0.32406e1*temperature+0.127e-1*term2-0.47723e-4*term3+0.58443e5*(-0.43679235e1+0.1169e-1*temperature-0.26185e-4*term2+0.70661e-7*term3)*term5)*(pressure-101325.0)))
}

/// Derivative of enthalpy w.r.t. temperature
/// 
/// Enthalpy, J/mol, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-393.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.04033898281\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of enthalpy w.r.t. temperature, J/mol/K, or an error
pub fn enthalpy_d_temperature(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>393.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,393.15] K range for enthalpy correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.04033898281 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.04033898281] mol/mol range for enthalpy correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for enthalpy correlation",pressure));
	}
	let term1=temperature-0.27315e3;
	let term2=term1*term1;
	let term3=0.4042772e2*x_nacl+0.1801528e2;
	let term4=1.0/term3*x_nacl;
	let term5=x_nacl*x_nacl;
	let term6=term3*term3;
	Ok((0.4042772e-1*x_nacl+0.1801528e-1)*(0.449434050e4-0.1070e1*temperature+0.12e-1*term2-0.58443e2*(0.3195794379e5-0.883466e2*temperature+0.64182e0*term2-0.1163646884e7*term4+0.9490064212e8/term6*term5+0.1137068177e5*term1*term4)*term4+0.1e-5*(-0.10178610e2+0.254e-1*temperature-0.143169e-3*term2+0.58443e5*(0.2599486550e-1-0.52370e-4*temperature+0.211983e-6*term2)*term4)*(pressure-101325.0)))
}

/// Derivative of enthalpy w.r.t. pressure
/// 
/// Enthalpy, J/mol, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-393.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.04033898281\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of enthalpy w.r.t. pressure, J/mol/Pa, or an error
pub fn enthalpy_d_pressure(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>393.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,393.15] K range for enthalpy correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.04033898281 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.04033898281] mol/mol range for enthalpy correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for enthalpy correlation",pressure));
	}
	let term1=temperature-0.27315e3;
	let term2=term1*term1;
	let term3=term2*term1;
	Ok((0.4042772e-1*x_nacl+0.1801528e-1)*(0.1881946590e-2-0.32406e-5*temperature+0.127e-7*term2-0.47723e-10*term3+0.58443e-1*(-0.43679235e1+0.1169e-1*temperature-0.26185e-4*term2+0.70661e-7*term3)/(0.4042772e2*x_nacl+0.1801528e2)*x_nacl))
}

/// Derivative of enthalpy w.r.t. NaCl mole fraction
/// 
/// Enthalpy, J/mol, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-393.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.04033898281\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of enthalpy w.r.t. NaCl mole fraction, J/mol, or an error
pub fn enthalpy_d_x_nacl(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>393.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,393.15] K range for enthalpy correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.04033898281 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.04033898281] mol/mol range for enthalpy correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for enthalpy correlation",pressure));
	}
	let term1=0.4042772e2*x_nacl+0.1801528e2;
	let term2=1.0/term1;
	let term3=term2*x_nacl;
	let term4=x_nacl*x_nacl;
	let term5=term1*term1;
	let term6=1.0/term5;
	let term7=term6*term4;
	let term8=term4*x_nacl;
	let term9=1.0/term5/term1;
	let term10=temperature-0.27315e3;
	let term11=term10*term10;
	let term12=term11*term10;
	let term13=-0.2161173520e7+0.1842024007e8*term3+0.9572823819e10*term7-0.2886581450e13*term9*term8+0.782607e4*temperature-0.441733e2*term11+0.21394e0*term12-0.1163646884e7*term10*term3+0.9490064212e8*term10*term7+0.5685340884e4*term11*term3;
	let term14=term6*x_nacl;
	let term15=term9*term4;
	let term16=term5*term5;
	let term17=pressure-101325.0;
	let term18=-0.43679235e1+0.1169e-1*temperature-0.26185e-4*term11+0.70661e-7*term12;
	Ok((0.4042772e-1*x_nacl+0.1801528e-1)*(-0.58443e2*term13*term2+0.2362717240e4*term13*term14-0.58443e2*(0.1842024007e8*term2+0.1840095933e11*term14-0.9433759232e13*term15+0.3500937199e15/term16*term8-0.1163646884e7*term10*term2+0.2368448746e9*term10*term14-0.7673233175e10*term10*term15+0.5685340884e4*term11*term2-0.2298453694e6*term11*term14)*term3+0.1e-5*(0.58443e5*term18*term2-0.2362717240e7*term18*term14)*term17)-0.4639703720e5+0.1698801094e3*temperature-0.2162883020e-1*term11+0.16171088e-3*term12-0.2362717240e1*term13*term3+0.4042772e-7*(0.1881946590e4-0.32406e1*temperature+0.127e-1*term11-0.47723e-4*term12+0.58443e5*term18*term3)*term17)
}

/// Calculate temperature, given composition, pressure and enthalpy
///
/// This calculation underlies the pressure-enthalpy flash
///
/// Enthalpy is a monotonic function of temperature; a simple 
/// bracketing and bisection approach is used where the amount
/// that is bisected is estimated from a limited Newton step,
/// with the derivative estimated from the bracket; a step
/// size limiter of 95% if the bracketed domain is imposed.
///
/// # Arguments:
/// * `enthalpy_value` - enthalpy, J/mol
/// * `pressure` - pressure, Pa
/// * `x_nacl` - NaCl mole fraction, mol/mol
///
/// # Returns
/// Temperature, K, or an error
pub fn solve_temperature_from_enthalpy(enthalpy_value: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	//get enthalpy at lower limit
	let mut t_min=273.15;
	let h_min=enthalpy(t_min,pressure,x_nacl)?;
	//get enthalpy at upper limit
	let mut t_max=393.15;
	let h_max=enthalpy(t_max,pressure,x_nacl)?;
	//check if bracketed
	if (h_min-enthalpy_value)*(h_max-enthalpy_value)>0.0 {
		return Err(format!("no solution for enthalpy of {} J/mol, pressure of {} Pa, NaCl mole fraction of {} mol/mol within valid temperature range of [273.15,393.15] K",enthalpy_value,pressure,x_nacl));
	}
	//initial guess: linear interpolation
	let mut temperature=t_min+(enthalpy_value-h_min)/(h_max-h_min)*(t_max-t_min);
	let max_iterations=100;
	let mut iteration=0;
	while iteration<max_iterations {
		//get enthalpty at current T
		let h=enthalpy(temperature,pressure,x_nacl)?;
		//check for convergence
		if f64::abs(h-enthalpy_value)<1e-10*f64::abs(enthalpy_value) {
			return Ok(temperature);
		}
		//get derivative
		let dh_dt=enthalpy_d_temperature(temperature,pressure,x_nacl)?;
		//determine newton step
		let mut delta_t=(enthalpy_value-h)/dh_dt;
		//check against brackets
		if delta_t>0.0 {
			let lim=temperature+0.95*(t_max-temperature);
			if temperature+delta_t>lim {
				delta_t=lim-temperature;
			}
		} else {
			let lim=temperature-0.95*(temperature-t_min);
			if temperature+delta_t<lim {
				delta_t=lim-temperature;
			}
		}
		//update bracket, temperature and iteration
		if delta_t>0.0 {
			t_min=temperature;
		} else {
			t_max=temperature;
		}
		temperature+=delta_t;
		iteration+=1;
	}
	Err(format!("could not converge to temperature for enthalpy of {} J/mol, pressure of {} Pa, NaCl mole fraction of {} mol/mol",enthalpy_value,pressure,x_nacl))
}

/// Entropy
/// 
/// Entropy, J/mol/K, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-393.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.04033898281\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Entropy, J/mol/K, or an error
pub fn entropy(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>393.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,393.15] K range for entropy correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.04033898281 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.04033898281] mol/mol range for entropy correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for entropy correlation",pressure));
	}
	let term1=temperature-0.27315e3;
	let term2=term1*term1;
	let term3=term2*term1;
	let term4=term2*term2;
	let term5=0.4042772e2*x_nacl+0.1801528e2;
	let term6=x_nacl/term5;
	let term7=x_nacl*x_nacl;
	let term8=term5*term5;
	let term9=term7/term8;
	Ok((0.4042772e-1*x_nacl+0.1801528e-1)*(-0.420171215e4+0.15383e2*temperature-0.2996e-1*term2+0.8193e-4*term3-0.1370e-6*term4-0.58443e2*(-0.74212030e4+0.85502109e6*term6-0.3374597238e9*term9+0.6178145850e11/term8/term5*term7*x_nacl+0.2562e2*temperature-0.1443e0*term2+0.5879e-3*term3-0.357145173e4*term1*term6+0.2746471295e6*term1*term9+0.177374505e2*term2*term6)*term6+0.1e-5*(0.317881150e1-0.11654e-1*temperature+0.61154e-4*term2-0.20696e-6*term3+0.58443e5*(-0.1249385010e-1+0.40054e-4*temperature-0.14193e-6*term2+0.33142e-9*term3)*term6)*(pressure-101325.0)))
}

/// Derivative of entropy w.r.t. temperature
/// 
/// Entropy, J/mol/K, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-393.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.04033898281\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of entropy w.r.t. temperature, J/mol/K^2, or an error
pub fn entropy_d_temperature(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>393.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,393.15] K range for entropy correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.04033898281 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.04033898281] mol/mol range for entropy correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for entropy correlation",pressure));
	}
	let term1=temperature-0.27315e3;
	let term2=term1*term1;
	let term3=0.4042772e2*x_nacl+0.1801528e2;
	let term4=x_nacl/term3;
	let term5=x_nacl*x_nacl;
	let term6=term3*term3;
	Ok((0.4042772e-1*x_nacl+0.1801528e-1)*(0.317501480e2-0.5992e-1*temperature+0.24579e-3*term2-0.5480e-6*term2*term1-0.58443e2*(0.104451090e3-0.2886e0*temperature+0.17637e-2*term2-0.357145173e4*term4+0.2746471295e6/term6*term5+0.354749010e2*term1*term4)*term4+0.1e-5*(-0.4506243020e-1+0.122308e-3*temperature-0.62088e-6*term2+0.58443e5*(0.1175903590e-3-0.28386e-6*temperature+0.99426e-9*term2)*term4)*(pressure-101325.0)))
}

/// Derivative of entropy w.r.t. pressure
/// 
/// Entropy, J/mol/K, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-393.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.04033898281\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of entropy w.r.t. pressure, J/mol/K/Pa, or an error
pub fn entropy_d_pressure(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>393.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,393.15] K range for entropy correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.04033898281 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.04033898281] mol/mol range for entropy correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for entropy correlation",pressure));
	}
	let term1=temperature-0.27315e3;
	let term2=term1*term1;
	let term3=term2*term1;
	Ok((0.4042772e-1*x_nacl+0.1801528e-1)*(0.317881150e-5-0.11654e-7*temperature+0.61154e-10*term2-0.20696e-12*term3+0.58443e-1*(-0.1249385010e-1+0.40054e-4*temperature-0.14193e-6*term2+0.33142e-9*term3)/(0.4042772e2*x_nacl+0.1801528e2)*x_nacl))
}

/// Derivative of entropy w.r.t. NaCl mole fraction
/// 
/// Entropy, J/mol/K, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-393.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.04033898281\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of entropy w.r.t. NaCl mole fraction, J/mol/K, or an error
pub fn entropy_d_x_nacl(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>393.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,393.15] K range for entropy correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.04033898281 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.04033898281] mol/mol range for entropy correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for entropy correlation",pressure));
	}
	let term1=0.4042772e2*x_nacl+0.1801528e2;
	let term2=1.0/term1;
	let term3=term2*x_nacl;
	let term4=x_nacl*x_nacl;
	let term5=term1*term1;
	let term6=1.0/term5;
	let term7=term6*term4;
	let term8=term4*x_nacl;
	let term9=term6/term1;
	let term10=temperature-0.27315e3;
	let term11=term10*term10;
	let term12=term11*term10;
	let term13=-0.74212030e4+0.85502109e6*term3-0.3374597238e9*term7+0.6178145850e11*term9*term8+0.2562e2*temperature-0.1443e0*term11+0.5879e-3*term12-0.357145173e4*term10*term3+0.2746471295e6*term10*term7+0.177374505e2*term11*term3;
	let term14=term6*x_nacl;
	let term15=term9*term4;
	let term16=term5*term5;
	let term17=pressure-101325.0;
	let term18=-0.1249385010e-1+0.40054e-4*temperature-0.14193e-6*term11+0.33142e-9*term12;
	let term19=term11*term11;
	Ok((0.4042772e-1*x_nacl+0.1801528e-1)*(-0.58443e2*term13*term2+0.2362717240e4*term13*term14-0.58443e2*(0.85502109e6*term2-0.7094860008e9*term14+0.2126298300e12*term15-0.7493050516e13/term16*term8-0.357145173e4*term10*term2+0.6936799095e6*term10*term14-0.2220671450e8*term10*term15+0.177374505e2*term11*term2-0.7170846823e3*term11*term14)*term3+0.1e-5*(0.58443e5*term18*term2-0.2362717240e7*term18*term14)*term17)-0.1698656423e3+0.6218996168e0*temperature-0.1211214491e-2*term11+0.3312243100e-5*term12-0.5538597640e-8*term19-0.2362717240e1*term13*term3+0.4042772e-7*(0.317881150e1-0.11654e-1*temperature+0.61154e-4*term11-0.20696e-6*term12+0.58443e5*term18*term3)*term17)
}

/// Calculate temperature, given composition, pressure and entropy
///
/// This calculation underlies the pressure-enthalpy flash
///
/// Within the validity of this equation, entropy 
/// is a monotonic function of temperature; a simple 
/// bracketing and bisection approach is used where the amount
/// that is bisected is estimated from a limited Newton step,
/// with the derivative estimated from the bracket; a step
/// size limiter of 95% if the bracketed domain is imposed.
///
/// # Arguments:
/// * `entropy_value` - entropy, J/mol/K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - NaCl mole fraction, mol/mol
///
/// # Returns
/// Temperature, K, or an error
pub fn solve_temperature_from_entropy(entropy_value: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	//get entropy at lower limit
	let mut t_min=273.15;
	let h_min=entropy(t_min,pressure,x_nacl)?;
	//get entropy at upper limit
	let mut t_max=393.15;
	let h_max=entropy(t_max,pressure,x_nacl)?;
	//check if bracketed
	if (h_min-entropy_value)*(h_max-entropy_value)>0.0 {
		return Err(format!("no solution for entropy of {} J/mol, pressure of {} Pa, NaCl mole fraction of {} mol/mol within valid temperature range of [273.15,393.15] K",entropy_value,pressure,x_nacl));
	}
	//initial guess: linear interpolation
	let mut temperature=t_min+(entropy_value-h_min)/(h_max-h_min)*(t_max-t_min);
	let max_iterations=100;
	let mut iteration=0;
	while iteration<max_iterations {
		//get enthalpty at current T
		let s=entropy(temperature,pressure,x_nacl)?;
		//check for convergence
		if f64::abs(s-entropy_value)<1e-10*f64::abs(entropy_value) {
			return Ok(temperature);
		}
		//get derivative
		let ds_dt=entropy_d_temperature(temperature,pressure,x_nacl)?;
		//determine newton step
		let mut delta_t=(entropy_value-s)/ds_dt;
		//check against brackets
		if delta_t>0.0 {
			let lim=temperature+0.95*(t_max-temperature);
			if temperature+delta_t>lim {
				delta_t=lim-temperature;
			}
		} else {
			let lim=temperature-0.95*(temperature-t_min);
			if temperature+delta_t<lim {
				delta_t=lim-temperature;
			}
		}
		//update bracket, temperature and iteration
		if delta_t>0.0 {
			t_min=temperature;
		} else {
			t_max=temperature;
		}
		temperature+=delta_t;
		iteration+=1;
	}
	Err(format!("could not converge to temperature for entropy of {} J/mol, pressure of {} Pa, NaCl mole fraction of {} mol/mol",entropy_value,pressure,x_nacl))
}

/// Density
/// 
/// Density, mol/m3, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-453.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.05159128949\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Density, mol/m3, or an error
pub fn density(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for density correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05159128949 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05159128949] mol/mol range for density correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for density correlation",pressure));
	}
	let term1=temperature*temperature;
	let term2=term1*temperature;
	let term3=pressure*pressure;
	let term4=f64::exp(0.1e1/(0.4042772000e2*x_nacl+0.1801528000e2)*(x_nacl*(term3*(0.9419635626e-15-0.5960391229e-17*temperature+0.2254029682e-19*term1-0.2750661641e-22*term2)-0.6896160478e-19*pressure*(term1-0.1389389905e4*temperature+0.5200208026e6)*(term1-0.6955982230e3*temperature+0.1360677953e6)*(temperature-0.1287510847e3)+0.6987534604e-14*(term1-0.1389348523e4*temperature+0.5200298103e6)*(term1-0.6956077280e3*temperature+0.1360524236e6)*(temperature-0.1287829628e3))+term3*(-0.1225741636e-22*term2+0.1004433984e-19*term1+0.2017682395e-15-0.2656051761e-17*temperature)-0.3073046462e-19*pressure*(term1-0.8553297432e3*temperature+0.2081777030e6)*(term1-0.5313454038e3*temperature+0.8146781323e5)*(temperature-0.8270640666e3)+0.3113764328e-14*(term1-0.8554353686e3*temperature+0.2082210058e6)*(term1-0.5313524700e3*temperature+0.8146583390e5)*(temperature-0.8269513755e3)));
	let term5=x_nacl*x_nacl;
	let term6=term1*term1;
	let term7=0.4042772e2*x_nacl+0.1801528e2;
	let term8=term7*term7;
	Ok(-0.7611403333e-1/term8/term7*(term6*(term5+0.8912340347e0*x_nacl+0.1985745261e0)+term2*(-0.6282263540e3*term5-0.9831791048e3*x_nacl-0.3133715895e3)+term1*(-0.3203958077e6*term5+0.2926413653e6*x_nacl+0.1941721516e6)+temperature*(0.2958078762e9*term5+0.1483450978e8*x_nacl-0.5220792038e8)-0.9602488223e11*term5-0.4103990941e11*x_nacl+0.7907375557e9)*term4)
}

/// Derivative of density w.r.t. temperature
/// 
/// Density, mol/m3, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-453.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.05159128949\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of density w.r.t. temperature, mol/m3/K, or an error
pub fn density_d_temperature(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for density correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05159128949 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05159128949] mol/mol range for density correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for density correlation",pressure));
	}
	let term1=temperature*temperature;
	let term2=pressure*pressure;
	let term3=term1-0.6955982230e3*temperature+0.1360677953e6;
	let term4=term1-0.1389389905e4*temperature+0.5200208026e6;
	let term5=temperature-0.1287510847e3;
	let term6=2.0*temperature;
	let term7=pressure*term4;
	let term8=term3*term5;
	let term9=term1-0.6956077280e3*temperature+0.1360524236e6;
	let term10=term1-0.1389348523e4*temperature+0.5200298103e6;
	let term11=temperature-0.1287829628e3;
	let term12=term9*term11;
	let term13=term1-0.5313454038e3*temperature+0.8146781323e5;
	let term14=term1-0.8553297432e3*temperature+0.2081777030e6;
	let term15=temperature-0.8270640666e3;
	let term16=pressure*term14;
	let term17=term13*term15;
	let term18=term1-0.5313524700e3*temperature+0.8146583390e5;
	let term19=term1-0.8554353686e3*temperature+0.2082210058e6;
	let term20=temperature-0.8269513755e3;
	let term21=term18*term20;
	let term22=1.0/(0.4042772000e2*x_nacl+0.1801528000e2);
	let term23=term1*temperature;
	let term24=f64::exp(term22*(x_nacl*(term2*(0.9419635626e-15-0.5960391229e-17*temperature+0.2254029682e-19*term1-0.2750661641e-22*term23)-0.6896160478e-19*term7*term8+0.6987534604e-14*term10*term12)+term2*(-0.1225741636e-22*term23+0.1004433984e-19*term1+0.2017682395e-15-0.2656051761e-17*temperature)-0.3073046462e-19*term17*term16+0.3113764328e-14*term19*term21));
	let term25=x_nacl*x_nacl;
	let term26=term25+0.8912340347e0*x_nacl+0.1985745261e0;
	let term27=term1*term1;
	let term28=-0.6282263540e3*term25-0.9831791048e3*x_nacl-0.3133715895e3;
	let term29=-0.3203958077e6*term25+0.2926413653e6*x_nacl+0.1941721516e6;
	let term30=0.2958078762e9*term25;
	let term31=0.1483450978e8*x_nacl;
	let term32=0.4042772e2*x_nacl+0.1801528e2;
	let term33=term32*term32;
	let term34=1.0/term33/term32;
	Ok(-0.7611403333e-1*term34*(term27*term26+term23*term28+term1*term29+temperature*(term30+term31-0.5220792038e8)-0.9602488223e11*term25-0.4103990941e11*x_nacl+0.7907375557e9)*term24*term22*(x_nacl*(term2*(-0.5960391229e-17+0.4508059364e-19*temperature-0.8251984923e-22*term1)-0.6896160478e-19*pressure*term4*term3-0.6896160478e-19*term7*(term6-0.6955982230e3)*term5-0.6896160478e-19*pressure*(term6-0.1389389905e4)*term8+0.6987534604e-14*term10*term9+0.6987534604e-14*term10*(term6-0.6956077280e3)*term11+0.6987534604e-14*(term6-0.1389348523e4)*term12)+term2*(-0.3677224908e-22*term1+0.2008867968e-19*temperature-0.2656051761e-17)-0.3073046462e-19*pressure*term14*term13-0.3073046462e-19*term16*(term6-0.5313454038e3)*term15-0.3073046462e-19*pressure*(term6-0.8553297432e3)*term17+0.3113764328e-14*term19*term18+0.3113764328e-14*term19*(term6-0.5313524700e3)*term20+0.3113764328e-14*(term6-0.8554353686e3)*term21)-0.7611403333e-1*term34*(0.4e1*term23*term26+0.3e1*term1*term28+0.2e1*temperature*term29+term30+term31-0.5220792038e8)*term24)
}

/// Derivative of density w.r.t. pressure
/// 
/// Density, mol/m3, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-453.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.05159128949\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of density w.r.t. pressure, mol/m3/Pa, or an error
pub fn density_d_pressure(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for density correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05159128949 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05159128949] mol/mol range for density correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for density correlation",pressure));
	}
	let term1=temperature*temperature;
	let term2=term1*temperature;
	let term3=0.9419635626e-15-0.5960391229e-17*temperature+0.2254029682e-19*term1-0.2750661641e-22*term2;
	let term4=(term1-0.6955982230e3*temperature+0.1360677953e6)*(temperature-0.1287510847e3);
	let term5=term1-0.1389389905e4*temperature+0.5200208026e6;
	let term6=-0.1225741636e-22*term2+0.1004433984e-19*term1+0.2017682395e-15-0.2656051761e-17*temperature;
	let term7=(term1-0.5313454038e3*temperature+0.8146781323e5)*(temperature-0.8270640666e3);
	let term8=term1-0.8553297432e3*temperature+0.2081777030e6;
	let term9=1.0/(0.4042772000e2*x_nacl+0.1801528000e2);
	let term10=pressure*pressure;
	let term11=f64::exp(term9*(x_nacl*(term10*term3-0.6896160478e-19*pressure*term5*term4+0.6987534604e-14*(term1-0.1389348523e4*temperature+0.5200298103e6)*(term1-0.6956077280e3*temperature+0.1360524236e6)*(temperature-0.1287829628e3))+term10*term6-0.3073046462e-19*pressure*term8*term7+0.3113764328e-14*(term1-0.8554353686e3*temperature+0.2082210058e6)*(term1-0.5313524700e3*temperature+0.8146583390e5)*(temperature-0.8269513755e3)));
	let term12=x_nacl*x_nacl;
	let term13=term1*term1;
	let term14=0.4042772e2*x_nacl+0.1801528e2;
	let term15=term14*term14;
	Ok(-0.7611403333e-1/term15/term14*(term13*(term12+0.8912340347e0*x_nacl+0.1985745261e0)+term2*(-0.6282263540e3*term12-0.9831791048e3*x_nacl-0.3133715895e3)+term1*(-0.3203958077e6*term12+0.2926413653e6*x_nacl+0.1941721516e6)+temperature*(0.2958078762e9*term12+0.1483450978e8*x_nacl-0.5220792038e8)-0.9602488223e11*term12-0.4103990941e11*x_nacl+0.7907375557e9)*term11*term9*(x_nacl*(0.2e1*pressure*term3-0.6896160478e-19*term5*term4)+0.2e1*pressure*term6-0.3073046462e-19*term8*term7))
}

/// Derivative of density w.r.t. NaCl mole fraction
/// 
/// Density, mol/m3, is calculated from:
///
/// Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024
/// 
/// It is valid in the range of \[273.15-453.15\] K, \[0-12e6\] Pa and x\[NaCl\]=\[0,0.05159128949\] mol/mol
///
/// #Arguments
/// * `temperature` - temperature, K
/// * `pressure` - pressure, Pa
/// * `x_nacl` - mole fraction of NaCl, mol/mol
///
/// # Returns
/// Derivative of density w.r.t. NaCl mole fraction, mol/m3, or an error
pub fn density_d_x_nacl(temperature: f64,pressure: f64, x_nacl: f64) -> Result<f64,String> {
	if temperature<273.15 || temperature>453.15 {
		return Err(format!("temperature of {} K is out of valid [273.15,453.15] K range for density correlation",temperature));
	}
	if x_nacl<0.0 || x_nacl>0.05159128949 {
		return Err(format!("NaCl mole fraction of {} mol/mol is out of valid [0,0.05159128949] mol/mol range for density correlation",x_nacl));
	}
	if pressure<0.0 || pressure>12e6 {
		return Err(format!("pressure of {} Pa is out of valid [0,12e6] Pa range for density correlation",pressure));
	}
	let term1=temperature*temperature;
	let term2=term1*temperature;
	let term3=pressure*pressure;
	let term4=term3*(0.9419635626e-15-0.5960391229e-17*temperature+0.2254029682e-19*term1-0.2750661641e-22*term2)-0.6896160478e-19*pressure*(term1-0.1389389905e4*temperature+0.5200208026e6)*(term1-0.6955982230e3*temperature+0.1360677953e6)*(temperature-0.1287510847e3)+0.6987534604e-14*(term1-0.1389348523e4*temperature+0.5200298103e6)*(term1-0.6956077280e3*temperature+0.1360524236e6)*(temperature-0.1287829628e3);
	let term5=0.4042772000e2*x_nacl+0.1801528000e2;
	let term6=1.0/term5;
	let term7=x_nacl*term4+term3*(-0.1225741636e-22*term2+0.1004433984e-19*term1+0.2017682395e-15-0.2656051761e-17*temperature)-0.3073046462e-19*pressure*(term1-0.8553297432e3*temperature+0.2081777030e6)*(term1-0.5313454038e3*temperature+0.8146781323e5)*(temperature-0.8270640666e3)+0.3113764328e-14*(term1-0.8554353686e3*temperature+0.2082210058e6)*(term1-0.5313524700e3*temperature+0.8146583390e5)*(temperature-0.8269513755e3);
	let term8=term5*term5;
	let term9=f64::exp(term6*term7);
	let term10=x_nacl*x_nacl;
	let term11=term1*term1;
	let term12=term11*(term10+0.8912340347e0*x_nacl+0.1985745261e0)+term2*(-0.6282263540e3*term10-0.9831791048e3*x_nacl-0.3133715895e3)+term1*(-0.3203958077e6*term10+0.2926413653e6*x_nacl+0.1941721516e6)+temperature*(0.2958078762e9*term10+0.1483450978e8*x_nacl-0.5220792038e8)-0.9602488223e11*term10-0.4103990941e11*x_nacl+0.7907375557e9;
	let term13=0.4042772e2*x_nacl+0.1801528e2;
	let term14=term13*term13;
	let term15=1.0/term14/term13;
	let term16=term14*term14;
	Ok(-0.7611403333e-1*term15*term12*term9*(term6*term4-0.4042772000e2/term8*term7)-0.7611403333e-1*term15*(term11*(0.2e1*x_nacl+0.8912340347e0)+term2*(-0.1256452708e4*x_nacl-0.9831791048e3)+term1*(-0.6407916154e6*x_nacl+0.2926413653e6)+(0.5916157524e9*x_nacl+0.1483450978e8)*temperature-0.1920497645e12*x_nacl-0.4103990941e11)*term9+0.9231350483e1/term16*term12*term9)
}


