/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2025  Luis M. Santos, M.D.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

pub mod serialization {
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub use serde_json::{from_str, to_string, to_string_pretty};

    ///
    /// Serialization macro which will take an object instance decorated with [Serialize] trait
    /// from serde and return the JSON string representation.
    ///
    /// ```
    /// pub use crate::rumtk_core::json::serialization::{Serialize};
    /// use crate::rumtk_core::strings::RUMString;
    /// use crate::rumtk_core::rumtk_serialize;
    ///
    /// #[derive(Serialize)]
    /// struct MyStruct {
    ///     hello: RUMString
    /// }
    ///
    /// let hw = MyStruct{hello: RUMString::from("World")};
    /// let hw_str = rumtk_serialize!(&hw, true).unwrap();
    ///
    /// assert!(hw_str.len() > 0, "Empty JSON string generated from the test struct!");
    ///
    /// ```
    #[macro_export]
    macro_rules! rumtk_serialize {
        ( $object:expr ) => {{
            use serde_json::to_string;
            to_string(&$object)
        }};
        ( $object:expr, $pretty:expr ) => {{
            use serde_json::{to_string, to_string_pretty};
            match $pretty {
                true => to_string_pretty(&$object),
                false => to_string(&$object),
            }
        }};
    }

    #[macro_export]
    macro_rules! rumtk_deserialize {
        ( $string:expr ) => {{
            use serde_json::from_str;
            from_str(&$string)
        }};
    }
}
