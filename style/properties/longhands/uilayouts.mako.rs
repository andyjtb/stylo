/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

${helpers.predefined_type(
    "shadow-color",
    "Color",
    "computed::Color::TRANSPARENT_BLACK",
    engines="servo",
    initial_specified_value="specified::Color::transparent()",
    ignored_when_colors_disabled=True,
    spec="nonstandard",
    affects="paint",
)}

${helpers.predefined_type(
    "shadow-offset-x",
    "Length",
    "computed::Length::zero()",
    engines="servo",
    spec="nonstandard",
    affects="paint",
)}

${helpers.predefined_type(
    "shadow-offset-y",
    "Length",
    "computed::Length::zero()",
    engines="servo",
    spec="nonstandard",
    affects="paint",
)}

${helpers.predefined_type(
    "shadow-radius",
    "NonNegativeLengthPercentage",
    "computed::NonNegativeLengthPercentage::zero()",
    engines="servo",
    spec="nonstandard",
    affects="paint",
)}

${helpers.single_keyword(
    "shadow-type",
    "normal none inset",
    engines="servo",
    animation_type="discrete",
    spec="nonstandard",
    affects="paint",
)}
