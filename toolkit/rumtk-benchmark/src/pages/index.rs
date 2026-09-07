/*
 *     rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 *     This toolkit aims to be reliable, simple, performant, and standards compliant.
 *     Copyright (C) 2026  Luis M. Santos, M.D.
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
use rumtk_web::components::form::form::form;
use rumtk_web::components::layouts::flex_row::flex_row;
use rumtk_web::components::layouts::spacer::vspacer;
use rumtk_web::components::text_card::text_card;
use rumtk_web::components::title::title;
use rumtk_web::defaults::*;
use rumtk_web::rumtk_web_params_map;
use rumtk_web::utils::*;

pub fn index(app_state: SharedAppState) -> RenderedPageComponentsResult {
    let title_params = rumtk_web_params_map!([(PARAMS_TITLE, "Intro")]);
    let title_intro = title(&[], title_params.get_inner(), app_state.clone())?;

    let text_card_params = rumtk_web_params_map!([(PARAMS_TYPE, "instructions")]);
    let text_card_intro = text_card(&[], text_card_params.get_inner(), app_state.clone())?;

    let spacer = vspacer(2)?;
    let basic_benchmark_params = rumtk_web_params_map!([
            (PARAMS_TYPE, "basic_benchmark"),
            (PARAMS_TITLE, "Basic Benchmark"),
            (PARAMS_TARGET, "basic_benchmark"),
            (PARAMS_SWAP_MODE, "outerHTML"),
            (PARAMS_ENDPOINT, "/api/benchmarks/basic")
        ]
    );
    let basic_benchmark = form(
        &[],
        basic_benchmark_params.get_inner(),
        app_state.clone()
    )?;
    let basic_benchamrk_box = flex_row("basic_benchmark_row", vec![basic_benchmark.to_string()])?;

    Ok(vec![title_intro.to_string(), text_card_intro.to_string(), spacer.to_string(),basic_benchamrk_box.to_string()])
}
