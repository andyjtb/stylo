/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

${helpers.predefined_type(
    "highlight-color",
    "ColorOrAuto",
    "generics::color::ColorOrAuto::Auto",
    engines="servo",
    ignored_when_colors_disabled=True,
    spec="nonstandard",
    affects="paint",
)}

${helpers.predefined_type(
    "highlight-text-color",
    "ColorOrAuto",
    "generics::color::ColorOrAuto::Auto",
    engines="servo",
    ignored_when_colors_disabled=True,
    spec="nonstandard",
    affects="paint",
)}

${helpers.predefined_type(
    "placeholder-color",
    "ColorOrAuto",
    "generics::color::ColorOrAuto::Auto",
    engines="servo",
    ignored_when_colors_disabled=True,
    spec="nonstandard",
    affects="paint",
)}

${helpers.predefined_type(
    "font-height",
    "NonNegativeLength",
    "computed::NonNegativeLength::new(13.0)",
    engines="servo",
    spec="nonstandard",
    affects="layout",
)}

${helpers.predefined_type(
    "font-kerning",
    "NonNegativeLength",
    "computed::NonNegativeLength::zero()",
    engines="servo",
    spec="nonstandard",
    affects="layout",
)}

${helpers.predefined_type(
    "src",
    "Image",
    engines="servo",
    initial_value="computed::Image::None",
    initial_specified_value="specified::Image::None",
    animation_type="discrete",
    spec="nonstandard",
    affects="paint",
)}

${helpers.predefined_type(
    "scrollbar-thumb-color",
    "ColorOrAuto",
    "generics::color::ColorOrAuto::Auto",
    engines="servo",
    ignored_when_colors_disabled=True,
    spec="nonstandard",
    affects="paint",
)}

${helpers.predefined_type(
    "scrollbar-track-color",
    "ColorOrAuto",
    "generics::color::ColorOrAuto::Auto",
    engines="servo",
    ignored_when_colors_disabled=True,
    spec="nonstandard",
    affects="paint",
)}
