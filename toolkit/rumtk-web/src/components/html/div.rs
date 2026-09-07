/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2025  Luis M. Santos, M.D. <lsantos@medicalmasses.com>
 * Copyright (C) 2025  Ethan Dixon
 * Copyright (C) 2025  MedicalMasses L.L.C. <contact@medicalmasses.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use crate::utils::types::SharedAppState;
use crate::{rumtk_web_get_config, sanitize_html, ComponentResult, RUMWebTemplate, RUMWebTemplateSafe};
use rumtk_core::strings::RUMString;

#[derive(RUMWebTemplate, Debug)]
#[template(
    source = "
        {% if custom_css_enabled %}
            <link href='/static/components/div.css' rel='stylesheet'>
        {% endif %}

        {% if id.is_empty() %}
            <div class='{{css_class}}'>{{contents|safe}}</div>
        {% else %}
            <div id={{id}} class='{{css_class}}'>{{contents|safe}}</div>
        {% endif %}
    ",
    ext = "html"
)]
pub struct Div {
    id: RUMString,
    contents: RUMString,
    css_class: RUMString,
    custom_css_enabled: bool,
}

impl RUMWebTemplateSafe for Div {}

pub fn div<T: ToString>(id: &str, contents: T, css_class: &str, state: SharedAppState) -> ComponentResult<Div> {
    let custom_css_enabled = rumtk_web_get_config!(state).flags.custom_css;

    let inner = contents.to_string();
    let contents = sanitize_html(&inner, false);

    Ok(Div {
        id: id.to_string(),
        contents,
        css_class: css_class.to_string(),
        custom_css_enabled
    })
}
