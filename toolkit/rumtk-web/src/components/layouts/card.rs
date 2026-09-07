/*
 *     rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 *     This toolkit aims to be reliable, simple, performant, and standards compliant.
 *     Copyright (C) 2026  Luis M. Santos, M.D. <lsantos@medicalmasses.com>
 *     Copyright (C) 2026  MedicalMasses L.L.C. <contact@medicalmasses.com>
 *
 *     This program is free software: you can redistribute it and/or modify
 *     it under the terms of the GNU General Public License as published by
 *     the Free Software Foundation, either version 3 of the License, or
 *     (at your option) any later version.
 *
 *     This program is distributed in the hope that it will be useful,
 *     but WITHOUT ANY WARRANTY; without even the implied warranty of
 *     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *     GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use crate::utils::types::RUMString;
use crate::{ComponentResult, RUMWebTemplate};
use rumtk_core::base::RUMVec;

#[derive(RUMWebTemplate, Debug, Clone)]
#[template(
    source = "
        <div {% if !id.is_empty() %}id='card-{{id}}'{% endif %} class='card-{{ css_class }}'>
            {% for e in contents %}
                {{ e|safe }}
            {% endfor %}
        </div>
    ",
    ext = "html"
)]
pub struct Card {
    id: RUMString,
    contents: RUMVec<RUMString>,
    css_class: RUMString,
}

pub fn card(id: &str, contents: RUMVec<RUMString>, css_class: &str) -> ComponentResult<Card> {
    Ok(Card {
        id: id.to_string(),
        contents,
        css_class: css_class.to_string(),
    })
}

