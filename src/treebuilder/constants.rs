//! HTML5 spec constants for tree building.

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

/// Void elements (self-closing, cannot have children).
pub static VOID_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ]
    .into_iter()
    .collect()
});

/// Heading elements.
pub static HEADING_ELEMENTS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["h1", "h2", "h3", "h4", "h5", "h6"].into_iter().collect());

/// Formatting elements (for adoption agency algorithm).
pub static FORMATTING_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike", "strong", "tt",
        "u",
    ]
    .into_iter()
    .collect()
});

/// Special elements that have specific parsing rules.
pub static SPECIAL_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "address", "applet", "area", "article", "aside", "base", "basefont", "bgsound",
        "blockquote", "body", "br", "button", "caption", "center", "col", "colgroup", "dd",
        "details", "dialog", "dir", "div", "dl", "dt", "embed", "fieldset", "figcaption", "figure",
        "footer", "form", "frame", "frameset", "h1", "h2", "h3", "h4", "h5", "h6", "head", "header",
        "hgroup", "hr", "html", "iframe", "img", "input", "keygen", "li", "link", "listing", "main",
        "marquee", "menu", "menuitem", "meta", "nav", "noembed", "noframes", "noscript", "object",
        "ol", "p", "param", "plaintext", "pre", "script", "search", "section", "select", "source",
        "style", "summary", "table", "tbody", "td", "template", "textarea", "tfoot", "th", "thead",
        "title", "tr", "track", "ul", "wbr",
    ]
    .into_iter()
    .collect()
});

/// Default scope terminators.
pub static DEFAULT_SCOPE_TERMINATORS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
    ]
    .into_iter()
    .collect()
});

/// Button scope terminators.
pub static BUTTON_SCOPE_TERMINATORS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = DEFAULT_SCOPE_TERMINATORS.clone();
    set.insert("button");
    set
});

/// List item scope terminators.
pub static LIST_ITEM_SCOPE_TERMINATORS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = DEFAULT_SCOPE_TERMINATORS.clone();
    set.insert("ol");
    set.insert("ul");
    set
});

/// Table scope terminators.
pub static TABLE_SCOPE_TERMINATORS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["html", "table", "template"].into_iter().collect());

/// Implied end tags.
pub static IMPLIED_END_TAGS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["dd", "dt", "li", "option", "optgroup", "p", "rb", "rp", "rt", "rtc"]
        .into_iter()
        .collect()
});

/// Elements that require foster parenting when in table context.
pub static TABLE_FOSTER_TARGETS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["table", "tbody", "tfoot", "thead", "tr"].into_iter().collect());

/// Elements that break out of foreign content (SVG/MathML).
pub static FOREIGN_BREAKOUT_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "b", "big", "blockquote", "body", "br", "center", "code", "dd", "div", "dl", "dt", "em",
        "embed", "h1", "h2", "h3", "h4", "h5", "h6", "head", "hr", "i", "img", "li", "listing",
        "menu", "meta", "nobr", "ol", "p", "pre", "ruby", "s", "small", "span", "strong", "strike",
        "sub", "sup", "table", "tt", "u", "ul", "var",
    ]
    .into_iter()
    .collect()
});

/// SVG tag name case adjustments.
pub static SVG_TAG_NAME_ADJUSTMENTS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [
        ("altglyph", "altGlyph"),
        ("altglyphdef", "altGlyphDef"),
        ("altglyphitem", "altGlyphItem"),
        ("animatecolor", "animateColor"),
        ("animatemotion", "animateMotion"),
        ("animatetransform", "animateTransform"),
        ("clippath", "clipPath"),
        ("feblend", "feBlend"),
        ("fecolormatrix", "feColorMatrix"),
        ("fecomponenttransfer", "feComponentTransfer"),
        ("fecomposite", "feComposite"),
        ("feconvolvematrix", "feConvolveMatrix"),
        ("fediffuselighting", "feDiffuseLighting"),
        ("fedisplacementmap", "feDisplacementMap"),
        ("fedistantlight", "feDistantLight"),
        ("feflood", "feFlood"),
        ("fefunca", "feFuncA"),
        ("fefuncb", "feFuncB"),
        ("fefuncg", "feFuncG"),
        ("fefuncr", "feFuncR"),
        ("fegaussianblur", "feGaussianBlur"),
        ("feimage", "feImage"),
        ("femerge", "feMerge"),
        ("femergenode", "feMergeNode"),
        ("femorphology", "feMorphology"),
        ("feoffset", "feOffset"),
        ("fepointlight", "fePointLight"),
        ("fespecularlighting", "feSpecularLighting"),
        ("fespotlight", "feSpotLight"),
        ("fetile", "feTile"),
        ("feturbulence", "feTurbulence"),
        ("foreignobject", "foreignObject"),
        ("glyphref", "glyphRef"),
        ("lineargradient", "linearGradient"),
        ("radialgradient", "radialGradient"),
        ("textpath", "textPath"),
    ]
    .into_iter()
    .collect()
});

/// SVG attribute case adjustments.
pub static SVG_ATTRIBUTE_ADJUSTMENTS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [
        ("attributename", "attributeName"),
        ("attributetype", "attributeType"),
        ("basefrequency", "baseFrequency"),
        ("baseprofile", "baseProfile"),
        ("calcmode", "calcMode"),
        ("clippathunits", "clipPathUnits"),
        ("diffuseconstant", "diffuseConstant"),
        ("edgemode", "edgeMode"),
        ("filterunits", "filterUnits"),
        ("glyphref", "glyphRef"),
        ("gradienttransform", "gradientTransform"),
        ("gradientunits", "gradientUnits"),
        ("kernelmatrix", "kernelMatrix"),
        ("kernelunitlength", "kernelUnitLength"),
        ("keypoints", "keyPoints"),
        ("keysplines", "keySplines"),
        ("keytimes", "keyTimes"),
        ("lengthadjust", "lengthAdjust"),
        ("limitingconeangle", "limitingConeAngle"),
        ("markerheight", "markerHeight"),
        ("markerunits", "markerUnits"),
        ("markerwidth", "markerWidth"),
        ("maskcontentunits", "maskContentUnits"),
        ("maskunits", "maskUnits"),
        ("numoctaves", "numOctaves"),
        ("pathlength", "pathLength"),
        ("patterncontentunits", "patternContentUnits"),
        ("patterntransform", "patternTransform"),
        ("patternunits", "patternUnits"),
        ("pointsatx", "pointsAtX"),
        ("pointsaty", "pointsAtY"),
        ("pointsatz", "pointsAtZ"),
        ("preservealpha", "preserveAlpha"),
        ("preserveaspectratio", "preserveAspectRatio"),
        ("primitiveunits", "primitiveUnits"),
        ("refx", "refX"),
        ("refy", "refY"),
        ("repeatcount", "repeatCount"),
        ("repeatdur", "repeatDur"),
        ("requiredextensions", "requiredExtensions"),
        ("requiredfeatures", "requiredFeatures"),
        ("specularconstant", "specularConstant"),
        ("specularexponent", "specularExponent"),
        ("spreadmethod", "spreadMethod"),
        ("startoffset", "startOffset"),
        ("stddeviation", "stdDeviation"),
        ("stitchtiles", "stitchTiles"),
        ("surfacescale", "surfaceScale"),
        ("systemlanguage", "systemLanguage"),
        ("tablevalues", "tableValues"),
        ("targetx", "targetX"),
        ("targety", "targetY"),
        ("textlength", "textLength"),
        ("viewbox", "viewBox"),
        ("viewtarget", "viewTarget"),
        ("xchannelselector", "xChannelSelector"),
        ("ychannelselector", "yChannelSelector"),
        ("zoomandpan", "zoomAndPan"),
    ]
    .into_iter()
    .collect()
});

/// Check if an element is a void element.
#[inline]
pub fn is_void_element(name: &str) -> bool {
    VOID_ELEMENTS.contains(name)
}

/// Check if an element is a formatting element.
#[inline]
pub fn is_formatting_element(name: &str) -> bool {
    FORMATTING_ELEMENTS.contains(name)
}

/// Check if an element is a special element.
#[inline]
pub fn is_special_element(name: &str) -> bool {
    SPECIAL_ELEMENTS.contains(name)
}

/// Check if an element is a heading element.
#[inline]
pub fn is_heading_element(name: &str) -> bool {
    HEADING_ELEMENTS.contains(name)
}
