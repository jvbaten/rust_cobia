use crate::number_formatter;
use arrayvec::ArrayVec;
use cobia::*;

/// Converts a CAPE-OPEN dimensionality vector into a compact unit string.
///
/// The input is interpreted as exponents for [`cape_open_1_2::CapeDimensionality`].
/// Positive exponents are rendered in the numerator,
/// negative exponents in the denominator.
///
/// Notes:
/// - `CapeDifferenceFlag` is intentionally ignored.
/// - Some common dimensional combinations are replaced with canonical symbols
///   (for example `N`, `Pa`, `J`, `W`, `Pa·s`).
///
/// # Parameters
/// - `dimensionality`: A type that can be referenced as a slice of `f64` values, representing the exponents of the base dimensions.
///
/// # Returns
/// A string representation of the unit corresponding to the given dimensionality.
pub(crate) fn as_string<T>(dimensionality: T) -> String
where
    T: AsRef<[f64]>,
{
    const EPSILON: f64 = 1e-10;
    let mut pos = ArrayVec::<
        (&'static str, f64),
        { cape_open_1_2::CapeDimensionality::CapeDimensionalityLength as usize },
    >::new();
    let mut neg = ArrayVec::<
        (&'static str, f64),
        { cape_open_1_2::CapeDimensionality::CapeDimensionalityLength as usize },
    >::new();
    let dimensionality = dimensionality.as_ref();
    for (index, dim) in cape_open_1_2::CapeDimensionality::iter().enumerate() {
        if index >= dimensionality.len() {
            break;
        }
        let (abbreviation, first) = match dim {
            cape_open_1_2::CapeDimensionality::CapeMeter => ("m", false),
            cape_open_1_2::CapeDimensionality::CapeKilogram => ("kg", true), //nice to have this at the front
            cape_open_1_2::CapeDimensionality::CapeSecond => ("s", false),
            cape_open_1_2::CapeDimensionality::CapeAmpere => ("A", false),
            cape_open_1_2::CapeDimensionality::CapeKelvin => ("K", false),
            cape_open_1_2::CapeDimensionality::CapeMole => ("mol", false),
            cape_open_1_2::CapeDimensionality::CapeCandela => ("cd", false),
            cape_open_1_2::CapeDimensionality::CapeRadian => ("rad", false),
            cape_open_1_2::CapeDimensionality::CapeDifferenceFlag => {
                //ignored for this implementation, used for unit conversions
                continue;
            }
            cape_open_1_2::CapeDimensionality::CapeDimensionalityLength => {
                //if we did not exit here, the dimnenality array is too large
                return "[?]".to_string();
            }
        };
        let val = dimensionality[index];
        if val < -EPSILON {
            if first {
                neg.insert(0, (abbreviation, -val));
            } else {
                neg.push((abbreviation, -val));
            }
        } else if val > EPSILON {
            if first {
                pos.insert(0, (abbreviation, val));
            } else {
                pos.push((abbreviation, val));
            }
        }
    }
    if pos.is_empty() && neg.is_empty() {
        return "".to_string();
    }
    let mut s = String::new();
    if pos.is_empty() {
        s.push('1');
    } else {
        let mut first = true;
        for (dim, val) in pos.iter() {
            if !first {
                s.push('·');
            }
            first = false;
            s.push_str(dim);
            if f64::abs(*val - 1.0) > EPSILON {
                s.push_str(&format!(
                    "<sup>{}</sup>",
                    number_formatter::format_number(Some(*val))
                ));
            }
        }
    }
    if !neg.is_empty() {
        s.push('/');
        let use_brackets = neg.len() > 1;
        if use_brackets {
            s.push('(');
        }
        let mut first = true;
        for (dim, val) in neg.iter() {
            if !first {
                s.push('·');
            }
            first = false;
            s.push_str(dim);
            if f64::abs(*val - 1.0) > EPSILON {
                s.push_str(&format!(
                    "<sup>{}</sup>",
                    number_formatter::format_number(Some(*val))
                ));
            }
        }
        if use_brackets {
            s.push(')');
        }
    }
    //replace some common units with more compact representations
    match s.as_str() {
        "kg·m/s<sup>2</sup>" => "N".to_string(),
        "kg·m/s<sup>3</sup>" => "W".to_string(),
        "kg/(m·s<sup>2</sup>)" => "Pa".to_string(),
        "kg/(m·s)" => "Pa·s".to_string(),
        "kg·m<sup>2</sup>/s<sup>2</sup>" => "J".to_string(),
        "kg·m<sup>2</sup>/s<sup>3</sup>" => "W".to_string(),
        _ => s,
    }
}
