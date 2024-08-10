
pub use rumtk_core::search::rumtk_search::*;

/**************************** Globals **************************************/

/**************************** Constants**************************************/

//pub const REGEX_V2_SEARCH_DEFAULT: &str = r"(?<segment>\w{3}).*(?<field>-?\d+).*.(?<component>-?\d+)|\w{3}.*\((?<segment_group>\d+)\).*|.*\d+\((?<sub_field>\d+)\)";
pub const REGEX_V2_SEARCH_DEFAULT: &str = r".*\d+\((?<sub_field>\d+)\)|\w{3}.*\((?<segment_group>\d+)\).*|(?<segment>\w{3}).*(?<field>-?\d+).*.(?<component>-?\d+)";

/**************************** Types *****************************************/

/**************************** Traits ****************************************/

/**************************** Helpers ***************************************/
