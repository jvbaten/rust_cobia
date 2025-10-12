//! # Distillation shortcut unit operation
//!
//! This is an example unit operation implementation
//! featuring a shortcut distillation unit operation
//! using the Fenske-Underwood-Gilliland-Kirkbride
//! approach.
//!
//! ## Fenske
//!
//! The Fenske equation (Fenske, "Fractionation of Straight-Run Pennsylvania Gasoline",
//! Industrial and Engineering Chemistry, Vol. 24: 482, 1932) is used
//! to estimate distillate and bottoms composition and the minimum number
//! of stages.
//! 
//! From the slate of components, two components are
//! selected that are called the Light Key (LK)
//! component, and Heavy Key (HK) component. The
//! desired recovery is specified for the LK:
//!
//! <math>
//!     <msub><ms>r</ms><mtext>LK</mtext></msub>
//!     <mo>=</mo>
//!     <mfrac>
//!     <msub><ms>f</ms><mrow><mtext>LK</mtext><mo>,</mo><mtext>dist</mtext></mrow></msub>
//!     <msub><ms>f</ms><mrow><mtext>LK</mtext><mo>,</mo><mtext>feed</mtext></mrow></msub>
//!     </mfrac>
//! </math>
//!
//! and for the heavy component recovery
//!
//! <math>
//!     <msub><ms>r</ms><mtext>HK</mtext></msub>
//!     <mo>=</mo>
//!     <mfrac>
//!     <msub><ms>f</ms><mrow><mtext>HK</mtext><mo>,</mo><mtext>bot</mtext></mrow></msub>
//!     <msub><ms>f</ms><mrow><mtext>HK</mtext><mo>,</mo><mtext>feed</mtext></mrow></msub>
//!     </mfrac>
//! </math>
//!
//! Initial distillate and bottom compositions are estimated.
//! 
//! The separation is based on calculation of the 
//! relative volatility for each component:
//!
//!  <math>
//!     <msub><ms>&alpha;</ms><mtext>i</mtext></msub>
//!     <mo>=</mo>
//!     <mfrac>
//!         <msub><ms>K</ms><mtext>i</mtext></msub>
//!         <msub><ms>K</ms><mtext>HK</mtext></msub>
//!     </mfrac>
//! </math>
//!
//! where the K value for each compound is calculated for the dew point (DP) at
//! distillate conditions, and bubble point (BP) at bottoms conditions, and the 
//! average K value is given by 
//!
//! <math>
//!     <msub><ms>K</ms><mtext>i</mtext></msub>
//!     <mo>=</mo>
//!     <msqrt>
//!         <msub><ms>K</ms><mrow><mtext>i</mtext><mo>,</mo><mtext>DP</mtext><mo>,</mo><mtext>dist</mtext></mrow></msub>
//!         <msub><ms>K</ms><mrow><mtext>i</mtext><mo>,</mo><mtext>BP</mtext><mo>,</mo><mtext>bot</mtext></mrow></msub>
//!     </msqrt>
//! </math>
//!
//! The minimum number of stages follows from:
//! 
//!  <math>
//!     <msub><ms>N</ms><mtext>min</mtext></msub>
//!     <mo>=</mo>
//!     <mfrac>
//!     <mrow><mtext>ln</mtext><mo>(</mo>
//!       <mfrac>
//!        <mrow>
//!        <msub><ms>f</ms><mrow><mtext>LK</mtext><mo>,</mo><mtext>dist</mtext></mrow></msub>
//!        <msub><ms>f</ms><mrow><mtext>HK</mtext><mo>,</mo><mtext>bot</mtext></mrow></msub>
//!        </mrow>
//!        <mrow>
//!        <msub><ms>f</ms><mrow><mtext>HK</mtext><mo>,</mo><mtext>dist</mtext></mrow></msub>
//!        <msub><ms>f</ms><mrow><mtext>LK</mtext><mo>,</mo><mtext>bot</mtext></mrow></msub>
//!        </mrow>
//!       </mfrac>
//!       <mo>)</mo></mrow>
//!     <mrow><mtext>ln</mtext><mo>(</mo><msub><ms>&alpha;</ms><mtext>LK</mtext></msub><mo>)</mo></mrow>
//!     </mfrac>
//! </math>
//!
//! The bottom component flow rates are:
//!
//!  <math>
//!     <msub><ms>f</ms><mrow><mtext>i</mtext><mo>,</mo><mtext>bot</mtext></mrow></msub>
//!     <mo>=</mo>
//!       <mfrac>
//!        <msub><ms>f</ms><mrow><mtext>i</mtext><mo>,</mo><mtext>feed</mtext></mrow></msub>
//!        <mrow><mn>1</mn><mo>+</mo>
//!        <mo>(</mo><mfrac>
//!            <msub><ms>f</ms><mrow><mtext>HK</mtext><mo>,</mo><mtext>dist</mtext></mrow></msub>
//!            <msub><ms>f</ms><mrow><mtext>HK</mtext><mo>,</mo><mtext>bot</mtext></mrow></msub>
//!        </mfrac><mo>)</mo>
//!        <mmultiscripts><ms>&alpha;</ms><mtext>i</mtext><msub><ms>N</ms><mtext>min</mtext></msub></mmultiscripts>
//!        </mrow>
//!       </mfrac>
//! </math>
//!
//! and the distillate component flow rates are:
//!
//! <math>
//!   <msub><ms>f</ms><mrow><mtext>i</mtext><mo>,</mo><mtext>dist</mtext></mrow></msub>
//!   <mo>=</mo><msub><ms>f</ms><mrow><mtext>i</mtext><mo>,</mo><mtext>feed</mtext></mrow></msub>
//!   <mo>-</mo><msub><ms>f</ms><mrow><mtext>i</mtext><mo>,</mo><mtext>bot</mtext></mrow></msub>
//! </math>
//!
//! The unit operation operates isobarically, so product pressure equals feed pressure. The distillate 
//! product is at the dew point temperature for the distillate composition, and the bottoms
//! product it at the bubble point temperature for the distillate composition. 
//!
//! Given the new distillate temperature and composition, and the new bottoms temperature
//! and composition, the K values and relative volatilities are updated, and the above 
//! calculation is repeated until converged values are obtained.
//!
//! ## Underwood
//!
//! The Underwood equation (Underwood, "Fractional distillation of
//! multi-component mixtures. Calculation of minimum reflux ratio", J. Inst. 
//! Petrol., 32, 614, 1946) is used to estimate the minimum reflux ratio.
//! 
//! The minimum reflux ratio is determined from
//! 
//! <math>
//!     <msub><ms>R</ms><mtext>min</mtext></msub>
//!     <mo>=</mo>
//!     <mo>&sum;</mo>
//!       <mfrac>
//!        <mrow>
//!            <msub><ms>&alpha;</ms><mtext>i</mtext></msub>
//!            <msub><ms>x</ms><mrow><mtext>i</mtext><mo>,</mo><mtext>dist</mtext></mrow></msub>
//!        </mrow>
//!        <mrow>
//!            <msub><ms>&alpha;</ms><mtext>i</mtext></msub>
//!            <mo>-</mo>
//!            <ms>&theta;</ms>
//!        </mrow>
//!       </mfrac>
//!       <mo>-</mo>
//!       <mn>1</mn>
//! </math>
//!
//! where Underwood parameter <math>&theta;</math> is the root of equation
//! 
//! <math>
//!     <mn>1</mn>
//!		<mo>-</mo>
//!     <ms>q</ms>
//!     <mo>=</mo>
//!     <mo>&sum;</mo>
//!     <mfrac>
//!      <mrow>
//!          <msub><ms>&alpha;</ms><mtext>i</mtext></msub>
//!          <msub><ms>x</ms><mrow><mtext>i</mtext><mo>,</mo><mtext>feed</mtext></mrow></msub>
//!      </mrow>
//!      <mrow>
//!          <msub><ms>&alpha;</ms><mtext>i</mtext></msub>
//!          <mo>-</mo>
//!          <ms>&theta;</ms>
//!      </mrow>
//!     </mfrac>
//! </math>
//!
//! where q is the feed quality, defined as 
//! 
//! <math>
//!     <ms>q</ms>
//!     <mo>=</mo>
//!     <mfrac>
//!      <mrow>
//!          <msub><ms>h</ms><mtext>feed,DP</mtext></msub>
//!			<mo>-</mo>
//!          <msub><ms>h</ms><mtext>feed</mtext></msub>
//!      </mrow>
//!      <mrow>
//!          <msub><ms>h</ms><mtext>feed,DP</mtext></msub>
//!			<mo>-</mo>
//!          <msub><ms>h</ms><mtext>feed,BP</mtext></msub>
//!      </mrow>
//!     </mfrac>
//! </math>
//!
//! where <math><msub><ms>h</ms><mtext>feed,DP</mtext></msub></math>
//! is the enthalpy of the vapor phase at the dew point temperature
//! of the feed, and <math><msub><ms>h</ms><mtext>feed,BP</mtext></msub></math>
//! is the enthalpy of the liquid phase at the bubble point temperature
//! of the feed.
//!
//! There are multiple roots for <math><ms>&theta;</ms></math>, and the
//! root that is used is between 1 and <math><msub><ms>&alpha;</ms><mtext>LK</mtext></msub></math>.
//! In case there are &alpha; values that are in between 1 and
//! <math><msub><ms>&alpha;</ms><mtext>LK</mtext></msub></math>, a
//! warning is issued and the root that is closest to 1 is used.
//!
//! The unit does not check whether for retrograde flash results, 
//! which is outside the scope of this example.
//!
//! ## Gilliland
//!
//! The Gilliland equation (Gilliland, "Estimate the 
//! number of theoretical plates as a function of reflux ratio", 
//! Ind. Eng. Chem. 32(9): 1220-1223, 1940, doi 10.1021/ie50369a035)
//! is used to estimate the number of stages:
//! 
//! <math>
//!     <ms>Y</ms>
//!     <mo>=</mo>
//!     <mn>1</mn>
//!     <mo>-</mo>
//!     <mtext>exp</mtext>
//!        <mo>(</mo>
//!        <mo>(</mo>
//!        <mfrac>
//!            <mrow><mn>1</mn><mo>+</mo><mn>54.4</mn><ms>X</ms></mrow>
//!            <mrow><mn>11</mn><mo>+</mo><mn>117.2</mn><ms>X</ms></mrow>
//!        </mfrac>
//!        <mo>)</mo>
//!        <mo>(</mo>
//!        <mfrac>
//!            <mrow><ms>X</ms><mo>-</mo><mn>1</mn></mrow>
//!            <msqrt><ms>X</ms></msqrt>
//!        </mfrac>
//!        <mo>)</mo>
//!        <mo>)</mo>
//! </math>
//!
//! where
//!
//! <math>
//!     <ms>Y</ms>
//!     <mo>=</mo>
//!     <mfrac>
//!        <mrow><ms>N</ms><mo>-</mo><msub><ms>N</ms><mtext>min</mtext></msub></mrow>
//!        <mrow><ms>N</ms><mo>+</mo><mn>1</mn></mrow>
//!     </mfrac>
//! </math>
//! 
//! and
//! 
//! <math>
//!     <ms>X</ms>
//!     <mo>=</mo>
//!     <mfrac>
//!        <mrow><ms>R</ms><mo>-</mo><msub><ms>R</ms><mtext>min</mtext></msub></mrow>
//!        <mrow><ms>R</ms><mo>+</mo><mn>1</mn></mrow>
//!     </mfrac>
//! </math>
//! 
//! The reflux ratio <math>R</math> is determined from the minimum reflux ratio 
//! using an adjustable parameter:
//!
//! 
//! <math>
//!     <ms>R</ms>
//!     <mo>=</mo>
//!     <ms>k</ms>
//!     <msub><ms>R</ms><mtext>min</mtext></msub>
//! </math>
//!
//! where <math>k</math> must exceed 1. The default value for <math>k</math> is 1.15.
//!
//! ## Kirkbride
//!
//! The location of the feed stage is estimated from the Kirkbride correlation
//! (Kirkbride,  "Process Design Procedure for Multicomponent Fractionators",
//! Petr. Ref. 23(9), 321, 1944):
//! 
//! <math>
//!    <mfrac>
//!        <msub><ms>N</ms><mtext>top</mtext></msub>
//!        <msub><ms>N</ms><mtext>bot</mtext></msub>
//!    </mfrac>
//!     <mo>=</mo>
//!     <msup>
//!        <mrow>
//!            <mo>[</mo>
//!            <mfrac>
//!                <msub><ms>f</ms><mtext>bot</mtext></msub>
//!                <msub><ms>f</ms><mtext>dist</mtext></msub>
//!            </mfrac>
//!            <mfrac>
//!                <msub><ms>x</ms><mrow><mtext>HK</mtext><mo>,</mo><mtext>feed</mtext></mrow></msub>
//!                <msub><ms>x</ms><mrow><mtext>LK</mtext><mo>,</mo><mtext>feed</mtext></mrow></msub>
//!            </mfrac>
//!            <msup>
//!                <mrow>
//!                <mo>(</mo>
//!                <mfrac>
//!                    <msub><ms>x</ms><mrow><mtext>LK</mtext><mo>,</mo><mtext>bot</mtext></mrow></msub>
//!                    <msub><ms>x</ms><mrow><mtext>HK</mtext><mo>,</mo><mtext>dist</mtext></mrow></msub>
//!                </mfrac>
//!                <mo>)</mo>
//!                </mrow>
//!            <mn>2</mn>
//!            </msup>
//!            <mo>]</mo>
//!        </mrow>
//!        <mn>0.206</mn>
//!     </msup>
//! </math>
//! 
//! where
//! <math><ms>f</ms></math>
//! are molar flow rates, 
//! <math><ms>x</ms></math>
//! are mole fractions, and the number of trays above the feed 
//! <math><msub><ms>N</ms><mtext>top</mtext></msub></math>
//! and the number of trays below the feed 
//! <math><msub><ms>N</ms><mtext>bot</mtext></msub></math>
//! sum up to the number of trays <math><ms>N</ms></math>.
//!
//! # Parameters
//!
//! The unit operation has the following input parameters:
//! 
//! * HK: selection of the heavy key component,
//! * LK: selection of the light key component,
//! * <math><msub><ms>r</ms><mtext>LK</mtext></msub></math>, light component recovery, mol/mol,
//! * <math><msub><ms>r</ms><mtext>HK</mtext></msub></math>, heavy component recovery, mol/mol,
//! * <math><ms>k</ms></math>, factor of reflux ratio <math><ms>R</ms></math> above minimum reflux ratio <math><msub><ms>R</ms><mtext>min</mtext></msub></math>.
//! * the maximum number of iterations
//! * the convergence tolerance for the component flow rates relative to the total feed rate; also used for convergence of the Underwood equation
//!
//! The unit has the following output parameters:
//! 
//! * <math><ms>N</ms></math>, number of stages
//! * <math><ms>R</ms></math>, reflux ratio, mol/mol
//! * <math><msub><ms>N</ms><mtext>feed</mtext></msub></math>, feed stage location.
//!
//! # Installation and usage
//!
//! `cobiaRegister.exe distillation_shortcut_unit.dll`
//!
//! An installer for Windows is made available through the AmsterCHEM web site.
//!
//! After installation, the unit operation can be used in any CAPE-OPEN compliant flowsheeting environment.

use cobia;
mod shared_unit_data;
mod distillation_shortcut_unit;
mod port_collection;
mod material_port;
mod parameter_collection;
mod real_parameter;
mod string_parameter;
mod integer_parameter;
mod gui;

/// This function is called by functions generated by the `pmc_entry_points`
/// macro to check if the unit operation is registered for all users.
///
/// Registering a PMC for all users requires administrative privileges.
///
/// # Returns
/// * `false`
fn register_pmcs_for_all_users() -> bool {
	false
}

///A list of all the PMCs that need to be registered
static PMCS: &[cobia::PMCInfo] = &[cobia::pmc_info::<distillation_shortcut_unit::DistillationShortcutUnit>()];

cobia::pmc_entry_points!(PMCS, register_pmcs_for_all_users());
