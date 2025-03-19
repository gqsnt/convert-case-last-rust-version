//! Converts to and from various cases.
//!
//! # Basic Usage
//!
//! The most common use of this crate is to just convert a string into a
//! particular case, like snake, camel, or kebab.  You can use the [`ccase`]
//! macro to convert most string types into the new case.
//! ```
//! use convert_case::ccase;
//!
//! let s = "myVarName";
//! assert_eq!(ccase!(snake, s),  "my_var_name");
//! assert_eq!(ccase!(kebab, s),  "my-var-name");
//! assert_eq!(ccase!(pascal, s), "MyVarName");
//! assert_eq!(ccase!(title, s),  "My Var Name");
//! ```
//!
//! For more explicit conversion, import the [`Casing`] trait which adds methods
//! to string types that perform the conversion based on a variant of the [`Case`] enum.
//! ```
//! use convert_case::{Case, Casing};
//!
//! let s = "myVarName";
//! assert_eq!(s.to_case(Case::Snake),  "my_var_name");
//! assert_eq!(s.to_case(Case::Kebab),  "my-var-name");
//! assert_eq!(s.to_case(Case::Pascal), "MyVarName");
//! assert_eq!(s.to_case(Case::Title),  "My Var Name");
//! ```
//!
//! For a full list of cases, see [`Case`].
//!
//! # Splitting Conditions
//!
//! Case conversion starts by splitting a single identifier into a list of words.  The
//! condition for when to split and how to perform the split is defined by a [`Boundary`].
//!
//! By default, [`ccase`] and [`Casing::to_case`] will split identifiers at all locations
//! based on a list of [default boundaries](Boundary::defaults).
//!
//! ```
//! use convert_case::ccase;
//!
//! assert_eq!(ccase!(pascal, "hyphens-and_underscores"), "HyphensAndUnderscores");
//! assert_eq!(ccase!(pascal, "lowerUpper space"), "LowerUpperSpace");
//! assert_eq!(ccase!(snake, "HTTPRequest"), "http_request");
//! assert_eq!(ccase!(snake, "vector4d"), "vector_4_d")
//! ```
//!
//! Associated with each case is a [list of boundaries](Case::boundaries) that can be
//! used to split identifiers instead of the defaults.  We can use the following notation
//! with the [`ccase`] macro.
//! ```
//! use convert_case::ccase;
//!
//! assert_eq!(
//!     ccase!(snake -> title, "1999-25-01_family_photo.png"),
//!     "1999-25-01 Family Photo.png",
//! );
//! ```
//! Or we can use the [`from_case`](Casing::from_case) method on `Casing` before calling
//! `to_case`.
//! ```
//! use convert_case::{Case, Casing};
//!
//! assert_eq!(
//!     "John McCarthy".from_case(Case::Title).to_case(Case::Snake),
//!     "john_mccarthy",
//! );
//! ```
//! You can specify exactly which boundaries are used with [`Casing::with_boundaries`].  See
//! the list of constants on [`Boundary`] for splitting conditions.
//! ```
//! use convert_case::{Boundary, Case, Casing};
//!
//! assert_eq!(
//!     "Vector4D".with_boundaries(&[Boundary::LOWER_DIGIT]).to_case(Case::Snake),
//!     "vector_4d",
//! );
//! ```
//!
//! # Other Behavior
//!
//! * removes trailing or duplicate delimiters
//! * acronyms aren't identified or preserved
//! * unicode?
//! * digits are funny
//! * symbols and non-cased values are ignored
//!
//! # Customizing Behavior
//!
//! Case conversion takes place in three steps:
//! 1. Splitting the identifier into a list of words
//! 2. Mutating the letter case of characters within each word
//! 3. Joining the words back into an identifier using a delimiter
//!
//! Those are defined by boundaries, patterns, and delimiters respectively.  Graphically:
//!
//! ```md
//! Identifier        Identifier
//!     |                 ^
//!     | boundaries      | delimiter
//!     V                 |
//!   Words ----------> Words
//!           pattern
//! ```
//!
//! ## Patterns
//!
//! How to change the case of letters across a list of words is called a _pattern_.
//! A pattern is a function that when passed a `&[&str]`, produces a
//! `Vec<String>`.  Inside the [`pattern`] module is a list of functions that are
//! used across all cases.  Although any function with type [`Pattern`](pattern::Pattern)
//! could be used.
//!
//! ## Boundaries
//!
//! The condition for splitting at part of an identifier, where to perform
//! the split, and if any characters are removed are defined by [boundaries](Boundary).
//! By default, identifies are split based on [`Boundary::defaults`].  This list
//! contains word boundaries that you would likely see after creating a multi-word
//! identifier of any case.
//!
//! Custom boundary conditions can created.  Commonly, you might split based on some
//! character or list of characters.  The [`Boundary::from_delim`] method builds
//! a boundary that splits on the presence of a string, and removes the string
//! from the final list of words.  You can also insantiate instances of boundaries
//! for very specific boundary conditions.  If you actually need to instantiate a
//! boundary condition from scratch, you should file an issue to let the author know.
//!
//! ## `Custom` Case variant
//!
//! In addition to a delimiter, the string to intersperse between words before concatenation,
//! the pattern and boundaries define a case.  [`Case::Custom`] is a struct enum variant with
//! exactly those three fields.  You could create your own case like so.
//! ```
//! use convert_case::{Case, Casing, Boundary, pattern};
//!
//! let dot_case = Case::Custom {
//!     boundaries: &[Boundary::from_delim(".")],
//!     pattern: pattern::lowercase,
//!     delim: ".",
//! };
//!
//! assert_eq!("AnimalFactoryFactory".to_case(dot_case), "animal.factory.factory");
//!
//! assert_eq!(
//!     "pd.options.mode.copy_on_write"
//!         .from_case(dot_case)
//!         .to_case(Case::Title),
//!     "Pd Options Mode Copy_on_write",
//! )
//! ```
//!
//! ## Converter
//!
//! The intent with case conversion is to use attributes from two cases.  From
//! the first case is how you split the identifier, and from the second is
//! how to mutate and join the words.  The [`Converter`] is used instead
//! to define the _conversion_ process, not a case directly.
//!
//! It has the same fields as case, but is exposed via a builder interface
//! and can be used to apply a conversion on a string directly, without
//! specifying all the parameters at the time of conversion.
//!
//! In the below example, we build a converter that maps the double colon
//! delimited module path in rust to a series of file directories.
//!
//! ```
//! use convert_case::{Case, Converter, Boundary, pattern};
//!
//! let modules_to_path = Converter::new()
//!     .set_boundaries(&[Boundary::from_delim("::")])
//!     .set_delim("/");
//!
//! assert_eq!(
//!     modules_to_path.convert("std::os::path"),
//!     "std/os/path",
//! );
//! ```
//!
//! # Old
//!
//! Provides a [`Case`] enum which defines a variety of cases to convert into.
//! Strings have implemented the [`Casing`] trait, which adds methods for
//! case conversion.
//!
//! You can convert strings into a case using the [`to_case`](Casing::to_case) method.
//! ```
//! use convert_case::{Case, Casing};
//!
//! assert_eq!("Ronnie James Dio", "ronnie james dio".to_case(Case::Title));
//! assert_eq!("ronnieJamesDio", "Ronnie_James_dio".to_case(Case::Camel));
//! assert_eq!("Ronnie-James-Dio", "RONNIE_JAMES_DIO".to_case(Case::Train));
//! ```
//!
//! By default, `to_case` will split along a set of default word boundaries, that is
//! * underscores `_`,
//! * hyphens `-`,
//! * spaces ` `,
//! * changes in capitalization from lowercase to uppercase `aA`,
//! * adjacent digits and letters `a1`, `1a`, `A1`, `1A`,
//! * and acroynms `AAa` (as in `HTTPRequest`).
//!
//! For more precision, the `from_case` method splits based on the word boundaries
//! of a particular case.  For example, splitting from snake case will only use
//! underscores as word boundaries.
//! ```
//! # use convert_case::{Case, Casing};
//! assert_eq!(
//!     "2020 04 16 My Cat Cali",
//!     "2020-04-16_my_cat_cali".to_case(Case::Title)
//! );
//! assert_eq!(
//!     "2020-04-16 My Cat Cali",
//!     "2020-04-16_my_cat_cali".from_case(Case::Snake).to_case(Case::Title)
//! );
//! ```
//!
//! This library can detect acronyms in camel-like strings.  It also ignores any leading,
//! trailing, or duplicate delimiters.
//! ```
//! # use convert_case::{Case, Casing};
//! assert_eq!("io_stream", "IOStream".to_case(Case::Snake));
//! assert_eq!("my_json_parser", "myJSONParser".to_case(Case::Snake));
//!
//! assert_eq!("weird_var_name", "__weird--var _name-".to_case(Case::Snake));
//! ```
//!
//! It also works non-ascii characters.  However, no inferences on the language itself is made.
//! For instance, the digraph `ij` in Dutch will not be capitalized, because it is represented
//! as two distinct Unicode characters.  However, `æ` would be capitalized.  Accuracy with unicode
//! characters is done using the `unicode-segmentation` crate, the sole dependency of this crate.
//! ```
//! # use convert_case::{Case, Casing};
//! assert_eq!("granat-äpfel", "GranatÄpfel".to_case(Case::Kebab));
//! assert_eq!("Перспектива 24", "ПЕРСПЕКТИВА24".to_case(Case::Title));
//!
//! // The example from str::to_lowercase documentation
//! let odysseus = "ὈΔΥΣΣΕΎΣ";
//! assert_eq!("ὀδυσσεύς", odysseus.to_case(Case::Lower));
//! ```
//!
//! By default, characters followed by digits and vice-versa are
//! considered word boundaries.  In addition, any special ASCII characters (besides `_` and `-`)
//! are ignored.
//! ```
//! # use convert_case::{Case, Casing};
//! assert_eq!("e_5150", "E5150".to_case(Case::Snake));
//! assert_eq!("10,000_days", "10,000Days".to_case(Case::Snake));
//! assert_eq!("HELLO, WORLD!", "Hello, world!".to_case(Case::Upper));
//! assert_eq!("One\ntwo\nthree", "ONE\nTWO\nTHREE".to_case(Case::Title));
//! ```
//!
//! You can also test what case a string is in.
//! ```
//! # use convert_case::{Case, Casing};
//! assert!( "css-class-name".is_case(Case::Kebab));
//! assert!(!"css-class-name".is_case(Case::Snake));
//! assert!(!"UPPER_CASE_VAR".is_case(Case::Snake));
//! ```
//!
//! # Note on Accuracy
//!
//! The `Casing` methods `from_case` and `to_case` do not fail.  Conversion to a case will always
//! succeed.  However, the results can still be unexpected.  Failure to detect any word boundaries
//! for a particular case means the entire string will be considered a single word.
//! ```
//! use convert_case::{Case, Casing};
//!
//! // Mistakenly parsing using Case::Snake
//! assert_eq!("My-kebab-var", "my-kebab-var".from_case(Case::Snake).to_case(Case::Title));
//!
//! // Converts using an unexpected method
//! assert_eq!("my_kebab_like_variable", "myKebab-like-variable".to_case(Case::Snake));
//! ```
//!
//! # Boundary Specificity
//!
//! It can be difficult to determine how to split a string into words.  That is why this case
//! provides the [`from_case`](Casing::from_case) functionality, but sometimes that isn't enough
//! to meet a specific use case.
//!
//! Say an identifier has the word `2D`, such as `scale2D`.  No exclusive usage of `from_case` will
//! be enough to solve the problem.  In this case we can further specify which boundaries to split
//! the string on.  `convert_case` provides some patterns for achieving this specificity.
//! We can specify what boundaries we want to split on using instances of the [`Boundary`] struct.
//! ```
//! use convert_case::{Boundary, Case, Casing};
//!
//! // Not quite what we want
//! assert_eq!(
//!     "scale_2_d",
//!     "scale2D"
//!         .from_case(Case::Camel)
//!         .to_case(Case::Snake)
//! );
//!
//! // Remove boundary from Case::Camel
//! assert_eq!(
//!     "scale_2d",
//!     "scale2D"
//!         .from_case(Case::Camel)
//!         .without_boundaries(&[Boundary::DIGIT_UPPER, Boundary::DIGIT_LOWER])
//!         .to_case(Case::Snake)
//! );
//!
//! // Write boundaries explicitly
//! assert_eq!(
//!     "scale_2d",
//!     "scale2D"
//!         .with_boundaries(&[Boundary::LOWER_DIGIT])
//!         .to_case(Case::Snake)
//! );
//! ```
//!
//! The `Casing` trait provides initial methods, but any subsequent methods that do not resolve
//! the conversion return a [`StateConverter`] struct.  It contains similar methods as `Casing`.
//!
//! ## Custom Boundaries
//!
//! `convert_case` provides a number of constants for boundaries associated with common cases.
//! But you can create your own boundary to split on other criteria.  For simple, delimiter
//! based splits, use [`Boundary::from_delim`].
//!
//! ```
//! # use convert_case::{Boundary, Case, Casing};
//! assert_eq!(
//!     "Coolers Revenge",
//!     "coolers.revenge"
//!         .with_boundaries(&[Boundary::from_delim(".")])
//!         .to_case(Case::Title)
//! )
//! ```
//!
//! For more complex boundaries, such as splitting based on the first character being a certain
//! symbol and the second is lowercase, you can instantiate a boundary directly.
//!
//! ```
//! # use convert_case::{Boundary, Case, Casing};
//! let at_then_letter = Boundary {
//!     name: "AtLetter",
//!     condition: |s, _| {
//!         s.get(0).map(|c| *c == "@") == Some(true)
//!             && s.get(1).map(|c| *c == c.to_lowercase()) == Some(true)
//!     },
//!     arg: None,
//!     start: 1,
//!     len: 0,
//! };
//! assert_eq!(
//!     "Name@ Domain",
//!     "name@domain"
//!         .with_boundaries(&[at_then_letter])
//!         .to_case(Case::Title)
//! )
//! ```
//!
//! To learn more about building a boundary from scratch, read the [`Boundary`] struct.
//!
//! # Custom Case
//!
//! Case has a special variant [`Case::Custom`] that exposes the three components necessary
//! for case conversion.  This allows you to define a custom case that behaves appropriately
//! in the `.to_case` and `.from_case` methods.
//!
//! A common example might be a "dot case" that has lowercase letters and is delimited by
//! periods.  We could define this as follows.
//! ```
//! use convert_case::{Case, Casing, pattern, Boundary};
//!
//! let dot_case = Case::Custom {
//!     boundaries: &[Boundary::from_delim(".")],
//!     pattern: pattern::lowercase,
//!     delim: ".",
//! };
//!
//! assert_eq!(
//!     "dot.case.var",
//!     "Dot case var".to_case(dot_case)
//! )
//! ```
//! And because we defined boundary conditions, this means `.from_case` should also behave as expected.
//! ```
//! # use convert_case::{Case, Casing, pattern, Boundary};
//! # let dot_case = Case::Custom {
//! #     boundaries: &[Boundary::from_delim(".")],
//! #     pattern: pattern::lowercase,
//! #     delim: ".",
//! # };
//! assert_eq!(
//!     "dotCaseVar",
//!     "dot.case.var".from_case(dot_case).to_case(Case::Camel)
//! )
//! ```
//!
//! # Converter Struct
//!
//! Case conversion takes place in two parts.  The first splits an identifier into a series of words,
//! and the second joins the words back together.  Each of these are steps are defined using the
//! `.from_case` and `.to_case` methods respectively.
//!
//! [`Converter`] is a struct that encapsulates the boundaries used for splitting and the pattern
//! and delimiter for mutating and joining.  The [`convert`](Converter::convert) method will
//! apply the boundaries, pattern, and delimiter appropriately.  This lets you define the
//! parameters for case conversion upfront.
//! ```
//! use convert_case::{Converter, pattern};
//!
//! let conv = Converter::new()
//!     .set_pattern(pattern::camel)
//!     .set_delim("_");
//!
//! assert_eq!(
//!     "my_Special_Case",
//!     conv.convert("My Special Case")
//! )
//! ```
//! For more details on how strings are converted, see the docs for [`Converter`].
//!
//! # Random Feature
//!
//! This feature adds two additional cases: [`Case::Random`] and [`Case::PseudoRandom`].
//! The `random` feature depends on the [`rand`](https://docs.rs/rand) crate.
//!
//! You can enable this feature by including the following in your `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! convert_case = { version = "^0.8.0", features = ["random"] }
//! ```
//!
//! # Associated Projects
//!
//! ## stringcase.org
//!
//! ## Command Line Utility `ccase`
//!
//! This library was developed for the purposes of a command line utility for converting
//! the case of strings and filenames.  You can check out
//! [`ccase` on Github](https://github.com/rutrum/ccase).

#![cfg_attr(not(test), no_std)]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

mod boundary;
mod case;
mod converter;

pub mod pattern;
pub use boundary::{split, Boundary};
pub use case::Case;
pub use converter::Converter;

/// Describes items that can be converted into a case.  This trait is used
/// in conjunction with the [`StateConverter`] struct which is returned from a couple
/// methods on `Casing`.
pub trait Casing<T: AsRef<str>> {
    /// Convert the string into the given case.  It will reference `self` and create a new
    /// `String` with the same pattern and delimeter as `case`.  It will split on boundaries
    /// defined at [`Boundary::defaults()`].
    /// ```
    /// use convert_case::{Case, Casing};
    ///
    /// assert_eq!(
    ///     "tetronimo-piece-border",
    ///     "Tetronimo piece border".to_case(Case::Kebab)
    /// );
    /// ```
    fn to_case(&self, case: Case) -> String;

    /// Start the case conversion by storing the boundaries associated with the given case.
    /// ```
    /// use convert_case::{Case, Casing};
    ///
    /// assert_eq!(
    ///     "2020-08-10_dannie_birthday",
    ///     "2020-08-10 Dannie Birthday"
    ///         .from_case(Case::Title)
    ///         .to_case(Case::Snake)
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    fn from_case(&self, case: Case) -> StateConverter<T>;

    /// Creates a `StateConverter` struct initialized with the boundaries
    /// provided.
    /// ```
    /// use convert_case::{Boundary, Case, Casing};
    ///
    /// assert_eq!(
    ///     "e1_m1_hangar",
    ///     "E1M1 Hangar"
    ///         .with_boundaries(&[Boundary::DIGIT_UPPER, Boundary::SPACE])
    ///         .to_case(Case::Snake)
    /// );
    /// ```
    fn with_boundaries(&self, bs: &[Boundary]) -> StateConverter<T>;

    /// Creates a `StateConverter` struct initialized without the boundaries
    /// provided.
    /// ```
    /// use convert_case::{Boundary, Case, Casing};
    ///
    /// assert_eq!(
    ///     "2d_transformation",
    ///     "2dTransformation"
    ///         .without_boundaries(&Boundary::digits())
    ///         .to_case(Case::Snake)
    /// );
    /// ```
    fn without_boundaries(&self, bs: &[Boundary]) -> StateConverter<T>;

    /// Determines if `self` is of the given case.  This is done simply by applying
    /// the conversion and seeing if the result is the same.
    /// ```
    /// use convert_case::{Case, Casing};
    ///
    /// assert!( "kebab-case-string".is_case(Case::Kebab));
    /// assert!( "Train-Case-String".is_case(Case::Train));
    ///
    /// assert!(!"kebab-case-string".is_case(Case::Snake));
    /// assert!(!"kebab-case-string".is_case(Case::Train));
    /// ```
    fn is_case(&self, case: Case) -> bool;

    /// Consider removing
    fn detect_cases(&self) -> Vec<Case> {
        Case::deterministic_cases()
            .iter()
            .filter_map(|&c| self.is_case(c).then_some(c))
            .collect()
    }
}

impl<T: AsRef<str>> Casing<T> for T {
    fn to_case(&self, case: Case) -> String {
        StateConverter::new(self).to_case(case)
    }

    fn with_boundaries(&self, bs: &[Boundary]) -> StateConverter<T> {
        StateConverter::new(self).with_boundaries(bs)
    }

    fn without_boundaries(&self, bs: &[Boundary]) -> StateConverter<T> {
        StateConverter::new(self).without_boundaries(bs)
    }

    fn from_case(&self, case: Case) -> StateConverter<T> {
        StateConverter::new(self).from_case(case)
    }

    fn is_case(&self, case: Case) -> bool {
        let digitless = self
            .as_ref()
            .chars()
            .filter(|x| !x.is_ascii_digit())
            .collect::<String>();

        digitless == digitless.to_case(case)
    }
}

/// Holds information about parsing before converting into a case.
///
/// This struct is used when invoking the `from_case` and `with_boundaries` methods on
/// `Casing`.  For a more fine grained approach to case conversion, consider using the [`Converter`]
/// struct.
/// ```
/// use convert_case::{Case, Casing};
///
/// let title = "ninety-nine_problems".from_case(Case::Snake).to_case(Case::Title);
/// assert_eq!("Ninety-nine Problems", title);
/// ```
pub struct StateConverter<'a, T: AsRef<str>> {
    s: &'a T,
    conv: Converter,
}

impl<'a, T: AsRef<str>> StateConverter<'a, T> {
    /// Only called by Casing function to_case()
    fn new(s: &'a T) -> Self {
        Self {
            s,
            conv: Converter::new(),
        }
    }

    /// Uses the boundaries associated with `case` for word segmentation.  This
    /// will overwrite any boundary information initialized before.  This method is
    /// likely not useful, but provided anyway.
    /// ```
    /// use convert_case::{Case, Casing};
    ///
    /// let name = "Chuck Schuldiner"
    ///     .from_case(Case::Snake) // from Casing trait
    ///     .from_case(Case::Title) // from StateConverter, overwrites previous
    ///     .to_case(Case::Kebab);
    /// assert_eq!("chuck-schuldiner", name);
    /// ```
    pub fn from_case(self, case: Case) -> Self {
        Self {
            conv: self.conv.from_case(case),
            ..self
        }
    }

    /// Overwrites boundaries for word segmentation with those provided.  This will overwrite
    /// any boundary information initialized before.  This method is likely not useful, but
    /// provided anyway.
    /// ```
    /// use convert_case::{Boundary, Case, Casing};
    ///
    /// let song = "theHumbling river-puscifer"
    ///     .from_case(Case::Kebab) // from Casing trait
    ///     .with_boundaries(&[Boundary::SPACE, Boundary::LOWER_UPPER]) // overwrites `from_case`
    ///     .to_case(Case::Pascal);
    /// assert_eq!("TheHumblingRiver-puscifer", song);  // doesn't split on hyphen `-`
    /// ```
    pub fn with_boundaries(self, bs: &[Boundary]) -> Self {
        Self {
            s: self.s,
            conv: self.conv.set_boundaries(bs),
        }
    }

    /// Removes any boundaries that were already initialized.  This is particularly useful when a
    /// case like `Case::Camel` has a lot of associated word boundaries, but you want to exclude
    /// some.
    /// ```
    /// use convert_case::{Boundary, Case, Casing};
    ///
    /// assert_eq!(
    ///     "2d_transformation",
    ///     "2dTransformation"
    ///         .from_case(Case::Camel)
    ///         .without_boundaries(&Boundary::digits())
    ///         .to_case(Case::Snake)
    /// );
    /// ```
    pub fn without_boundaries(self, bs: &[Boundary]) -> Self {
        Self {
            s: self.s,
            conv: self.conv.remove_boundaries(bs),
        }
    }

    /// Consumes the `StateConverter` and returns the converted string.
    /// ```
    /// use convert_case::{Boundary, Case, Casing};
    ///
    /// assert_eq!(
    ///     "ice-cream social",
    ///     "Ice-Cream Social".from_case(Case::Title).to_case(Case::Lower)
    /// );
    /// ```
    pub fn to_case(self, case: Case) -> String {
        self.conv.to_case(case).convert(self.s)
    }
}

#[cfg(not(feature = "random"))]
#[macro_export]
macro_rules! case {
    (snake) => {
        convert_case::Case::Snake
    };
    (constant) => {
        convert_case::Case::Constant
    };
    (upper_snake) => {
        convert_case::Case::UpperSnake
    };
    (ada) => {
        convert_case::Case::Ada;
    };
    (kebab) => {
        convert_case::Case::Kebab
    };
    (cobol) => {
        convert_case::Case::Cobol
    };
    (upper_kebab) => {
        convert_case::Case::UpperKebab
    };
    (train) => {
        convert_case::Case::Train
    };
    (flat) => {
        convert_case::Case::Flat
    };
    (upper_flat) => {
        convert_case::Case::UpperFlat
    };
    (pascal) => {
        convert_case::Case::Pascal
    };
    (upper_camel) => {
        convert_case::Case::UpperCamel
    };
    (camel) => {
        convert_case::Case::Camel
    };
    (lower) => {
        convert_case::Case::Lower
    };
    (upper) => {
        convert_case::Case::Upper
    };
    (title) => {
        convert_case::Case::Title
    };
    (sentence) => {
        convert_case::Case::Sentence
    };
    (alternating) => {
        convert_case::Case::Alternating
    };
    (toggle) => {
        convert_case::Case::Toggle
    };
}

#[cfg(feature = "random")]
#[macro_export]
macro_rules! case {
    (snake) => {
        convert_case::Case::Snake
    };
    (constant) => {
        convert_case::Case::Constant
    };
    (upper_snake) => {
        convert_case::Case::UpperSnake
    };
    (ada) => {
        convert_case::Case::Ada;
    };
    (kebab) => {
        convert_case::Case::Kebab
    };
    (cobol) => {
        convert_case::Case::Cobol
    };
    (upper_kebab) => {
        convert_case::Case::UpperKebab
    };
    (train) => {
        convert_case::Case::Train
    };
    (flat) => {
        convert_case::Case::Flat
    };
    (upper_flat) => {
        convert_case::Case::UpperFlat
    };
    (pascal) => {
        convert_case::Case::Pascal
    };
    (upper_camel) => {
        convert_case::Case::UpperCamel
    };
    (camel) => {
        convert_case::Case::Camel
    };
    (lower) => {
        convert_case::Case::Lower
    };
    (upper) => {
        convert_case::Case::Upper
    };
    (title) => {
        convert_case::Case::Title
    };
    (sentence) => {
        convert_case::Case::Sentence
    };
    (alternating) => {
        convert_case::Case::Alternating
    };
    (toggle) => {
        convert_case::Case::Toggle
    };
    (random) => {
        convert_case::Case::Random
    };
    (psuedo_random) => {
        convert_case::Case::PsuedoRandom
    };
}

#[macro_export]
macro_rules! ccase {
    ($case:ident, $e:expr) => {
        convert_case::Converter::new()
            .to_case(convert_case::case!($case))
            .convert($e)
    };
    ($from:ident -> $to:ident, $e:expr) => {
        convert_case::Converter::new()
            .from_case(convert_case::case!($from))
            .to_case(convert_case::case!($to))
            .convert($e)
    };
}

#[cfg(test)]
mod test {
    use super::*;

    use alloc::vec;
    use alloc::vec::Vec;

    fn possible_cases(s: &str) -> Vec<Case> {
        Case::deterministic_cases()
            .iter()
            .filter(|&case| s.from_case(*case).to_case(*case) == s)
            .map(|c| *c)
            .collect()
    }

    #[test]
    fn lossless_against_lossless() {
        let examples = vec![
            (Case::Snake, "my_variable_22_name"),
            (Case::Constant, "MY_VARIABLE_22_NAME"),
            (Case::Ada, "My_Variable_22_Name"),
            (Case::Kebab, "my-variable-22-name"),
            (Case::Cobol, "MY-VARIABLE-22-NAME"),
            (Case::Train, "My-Variable-22-Name"),
            (Case::Pascal, "MyVariable22Name"),
            (Case::Camel, "myVariable22Name"),
            (Case::Lower, "my variable 22 name"),
            (Case::Upper, "MY VARIABLE 22 NAME"),
            (Case::Title, "My Variable 22 Name"),
            (Case::Sentence, "My variable 22 name"),
            (Case::Toggle, "mY vARIABLE 22 nAME"),
            (Case::Alternating, "mY vArIaBlE 22 nAmE"),
        ];

        for (case_a, str_a) in &examples {
            for (case_b, str_b) in &examples {
                assert_eq!(*str_a, str_b.from_case(*case_b).to_case(*case_a))
            }
        }
    }

    #[test]
    fn obvious_default_parsing() {
        let examples = vec![
            "SuperMario64Game",
            "super-mario64-game",
            "superMario64 game",
            "Super Mario 64_game",
            "SUPERMario 64-game",
            "super_mario-64 game",
        ];

        for example in examples {
            assert_eq!("super_mario_64_game", example.to_case(Case::Snake));
        }
    }

    #[test]
    fn multiline_strings() {
        assert_eq!("One\ntwo\nthree", "one\ntwo\nthree".to_case(Case::Title));
    }

    #[test]
    fn camel_case_acroynms() {
        assert_eq!(
            "xml_http_request",
            "XMLHttpRequest".from_case(Case::Camel).to_case(Case::Snake)
        );
        assert_eq!(
            "xml_http_request",
            "XMLHttpRequest"
                .from_case(Case::UpperCamel)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "xml_http_request",
            "XMLHttpRequest"
                .from_case(Case::Pascal)
                .to_case(Case::Snake)
        );
    }

    #[test]
    fn leading_tailing_delimeters() {
        assert_eq!(
            "leading_underscore",
            "_leading_underscore"
                .from_case(Case::Snake)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "tailing_underscore",
            "tailing_underscore_"
                .from_case(Case::Snake)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "leading_hyphen",
            "-leading-hyphen"
                .from_case(Case::Kebab)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "tailing_hyphen",
            "tailing-hyphen-"
                .from_case(Case::Kebab)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "tailing_hyphens",
            "tailing-hyphens-----"
                .from_case(Case::Kebab)
                .to_case(Case::Snake)
        );
    }

    #[test]
    fn double_delimeters() {
        assert_eq!(
            "many_underscores",
            "many___underscores"
                .from_case(Case::Snake)
                .to_case(Case::Snake)
        );
        assert_eq!(
            "many-underscores",
            "many---underscores"
                .from_case(Case::Kebab)
                .to_case(Case::Kebab)
        );
    }

    #[test]
    fn early_word_boundaries() {
        assert_eq!(
            "a_bagel",
            "aBagel".from_case(Case::Camel).to_case(Case::Snake)
        );
    }

    #[test]
    fn late_word_boundaries() {
        assert_eq!(
            "team_a",
            "teamA".from_case(Case::Camel).to_case(Case::Snake)
        );
    }

    #[test]
    fn empty_string() {
        for (case_a, case_b) in Case::all_cases()
            .into_iter()
            .zip(Case::all_cases().into_iter())
        {
            assert_eq!("", "".from_case(*case_a).to_case(*case_b));
        }
    }

    #[test]
    fn default_all_boundaries() {
        assert_eq!(
            "abc_abc_abc_abc_abc_abc",
            "ABC-abc_abcAbc ABCAbc".to_case(Case::Snake)
        );
        assert_eq!("8_a_8_a_8", "8a8A8".to_case(Case::Snake));
    }

    #[test]
    fn alternating_ignore_symbols() {
        assert_eq!("tHaT's", "that's".to_case(Case::Alternating));
    }

    mod is_case {
        use super::*;

        #[test]
        fn snake() {
            assert!("im_snake_case".is_case(Case::Snake));
            assert!(!"im_NOTsnake_case".is_case(Case::Snake));
        }

        #[test]
        fn kebab() {
            assert!("im-kebab-case".is_case(Case::Kebab));
            assert!(!"im_not_kebab".is_case(Case::Kebab));
        }

        #[test]
        fn lowercase_word() {
            for lower_case in [
                Case::Snake,
                Case::Kebab,
                Case::Flat,
                Case::Lower,
                Case::Camel,
            ] {
                assert!("lowercase".is_case(lower_case));
            }
        }

        #[test]
        fn uppercase_word() {
            for upper_case in [Case::Constant, Case::Cobol, Case::UpperFlat, Case::Upper] {
                assert!("UPPERCASE".is_case(upper_case));
            }
        }

        #[test]
        fn capital_word() {
            for capital_case in [
                Case::Ada,
                Case::Train,
                Case::Pascal,
                Case::Title,
                Case::Sentence,
            ] {
                assert!("Capitalcase".is_case(capital_case));
            }
        }

        #[test]
        fn underscores_not_kebab() {
            assert!(!"kebab-case".is_case(Case::Snake));
        }

        #[test]
        fn multiple_delimiters() {
            assert!(!"kebab-snake_case".is_case(Case::Snake));
            assert!(!"kebab-snake_case".is_case(Case::Kebab));
            assert!(!"kebab-snake_case".is_case(Case::Lower));
        }

        #[test]
        fn digits_ignored() {
            assert!("UPPER_CASE_WITH_DIGIT1".is_case(Case::Constant));

            assert!("transformation_2d".is_case(Case::Snake));

            assert!("Transformation2d".is_case(Case::Pascal));
            assert!("Transformation2D".is_case(Case::Pascal));

            assert!("transformation2D".is_case(Case::Camel));

            assert!(!"5isntPascal".is_case(Case::Pascal))
        }

        #[test]
        fn not_a_case() {
            for c in Case::all_cases() {
                assert!(!"hyphen-and_underscore".is_case(*c));
                assert!(!"Sentence-with-hyphens".is_case(*c));
                assert!(!"Sentence_with_underscores".is_case(*c));
            }
        }

        #[test]
        fn detect_single_word() {
            assert_eq!(
                "lowercase".detect_cases(),
                vec![
                    Case::Snake,
                    Case::Kebab,
                    Case::Flat,
                    Case::Camel,
                    Case::Lower,
                ],
            );
            assert_eq!(
                "UPPERCASE".detect_cases(),
                vec![Case::Constant, Case::Cobol, Case::UpperFlat, Case::Upper],
            );
            assert_eq!(
                "Capitalcase".detect_cases(),
                vec![
                    Case::Ada,
                    Case::Train,
                    Case::Pascal,
                    Case::Title,
                    Case::Sentence
                ],
            )
        }
    }

    #[test]
    fn remove_boundaries() {
        assert_eq!(
            "m02_s05_binary_trees.pdf",
            "M02S05BinaryTrees.pdf"
                .from_case(Case::Pascal)
                .without_boundaries(&[Boundary::UPPER_DIGIT])
                .to_case(Case::Snake)
        );
    }

    #[test]
    fn with_boundaries() {
        assert_eq!(
            "my-dumb-file-name",
            "my_dumbFileName"
                .with_boundaries(&[Boundary::UNDERSCORE, Boundary::LOWER_UPPER])
                .to_case(Case::Kebab)
        );
    }

    #[cfg(feature = "random")]
    #[test]
    fn random_case_boundaries() {
        for &random_case in Case::random_cases() {
            assert_eq!(
                "split_by_spaces",
                "Split By Spaces"
                    .from_case(random_case)
                    .to_case(Case::Snake)
            );
        }
    }

    #[test]
    fn multiple_from_case() {
        assert_eq!(
            "longtime_nosee",
            "LongTime NoSee"
                .from_case(Case::Camel)
                .from_case(Case::Title)
                .to_case(Case::Snake),
        )
    }

    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[test]
    fn detect_many_cases() {
        let lower_cases_vec = possible_cases(&"asef");
        let lower_cases_set = HashSet::from_iter(lower_cases_vec.into_iter());
        let mut actual = HashSet::new();
        actual.insert(Case::Lower);
        actual.insert(Case::Camel);
        actual.insert(Case::Snake);
        actual.insert(Case::Kebab);
        actual.insert(Case::Flat);
        assert_eq!(lower_cases_set, actual);

        let lower_cases_vec = possible_cases(&"asefCase");
        let lower_cases_set = HashSet::from_iter(lower_cases_vec.into_iter());
        let mut actual = HashSet::new();
        actual.insert(Case::Camel);
        assert_eq!(lower_cases_set, actual);
    }

    #[test]
    fn detect_each_case() {
        let s = "My String Identifier".to_string();
        for &case in Case::deterministic_cases() {
            let new_s = s.from_case(case).to_case(case);
            let possible = possible_cases(&new_s);
            assert!(possible.iter().any(|c| c == &case));
        }
    }

    // From issue https://github.com/rutrum/convert-case/issues/8
    #[test]
    fn accent_mark() {
        let s = "música moderna".to_string();
        assert_eq!("MúsicaModerna", s.to_case(Case::Pascal));
    }

    // From issue https://github.com/rutrum/convert-case/issues/4
    #[test]
    fn russian() {
        let s = "ПЕРСПЕКТИВА24".to_string();
        let _n = s.to_case(Case::Title);
    }
}
