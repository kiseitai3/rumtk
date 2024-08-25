/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2024  Luis M. Santos, M.D.
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


pub use rumtk_core::search::rumtk_search::*;

/**************************** Globals **************************************/

/**************************** Constants**************************************/

//pub const REGEX_V2_SEARCH_DEFAULT: &str = r"(?<segment>\w{3}).*(?<field>-?\d+).*.(?<component>-?\d+)|\w{3}.*\((?<segment_group>\d+)\).*|.*\d+\((?<sub_field>\d+)\)";
pub const REGEX_V2_SEARCH_DEFAULT: &str = r"(?<segment>\w{3})|(\((?<segment_group>\d+)\))|(?<field>-?\d+)|(\[(?<sub_field>\d+)\])|(.(?<component>-?\d+))";

/**************************** Types *****************************************/

/**************************** Traits ****************************************/

/**************************** Helpers ***************************************/
