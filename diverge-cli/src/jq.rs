//! Running a jq program over a JSON value: what a `run --jq` prints
//! through.

use std::fmt;

use jaq_core::data::JustLut;
use jaq_core::{Compiler, Ctx, Filter, Vars, load};
use jaq_json::Val;
use serde::Deserialize as _;
use serde_json::Value;

/// What a run of a program carries beside its value: its compiled
/// table and nothing more. A program has no input stream of its own —
/// the input of a run is the value it is given — so `input` and
/// `inputs` are names it does not have.
type Data = JustLut<Val>;

/// A jq program, compiled once and run over any number of values.
pub struct Program {
    /// The compiled filter.
    filter: Filter<Data>,
}

impl Program {
    /// `code` parsed and compiled with jaq's and jq's standard
    /// definitions; a program that does not parse, or does not compile
    /// — a name it defines nowhere — is the [`Error`], in jaq's own
    /// words.
    pub fn new(code: &str) -> Result<Program, Error> {
        let program = load::File { code, path: () };
        let loader = load::Loader::new(jaq_core::defs().chain(jaq_std::defs()).chain(jaq_json::defs()));
        let arena = load::Arena::default();
        let modules = loader.load(&arena, program).map_err(|errors| {
            Error(
                errors
                    .into_iter()
                    .map(|error| format!("{error:?}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })?;
        let filter = Compiler::default()
            .with_funs(jaq_core::funs().chain(jaq_std::funs()).chain(jaq_json::funs()))
            .compile(modules)
            .map_err(|errors| {
                Error(
                    errors
                        .into_iter()
                        .map(|error| format!("{error:?}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            })?;
        Ok(Program { filter })
    }

    /// Every value the program yields over `input`, in order, as the
    /// program yields it: each is taken as it is computed, so the first
    /// is had before the last is. A failure as the program runs is the
    /// [`Error`], in jaq's own words, after the values yielded before
    /// it; a program that halts ends there, with the [`Error`] saying
    /// so, and the process goes on. A value jaq yields that JSON has
    /// no way to write — a string that is not text, a number that is
    /// not finite — is the [`Error`] too.
    pub fn run<'a>(&'a self, input: Value) -> impl Iterator<Item = Result<Value, Error>> + 'a {
        // A JSON value is always one jaq takes; the error is carried
        // rather than supposed away.
        let (input, refused) = match Val::deserialize(input) {
            Ok(input) => (Some(input), None),
            Err(error) => (None, Some(Err(Error(error.to_string())))),
        };
        refused.into_iter().chain(input.into_iter().flat_map(move |input| {
            let ctx = Ctx::<Data>::new(&self.filter.lut, Vars::new([]));
            self.filter.id.run((ctx, input)).map(|item| match item {
                Ok(value) => serde_json::from_str(&value.to_string())
                    .map_err(|error| Error(format!("a value that is not JSON, `{value}`: {error}"))),
                Err(exception) => Err(match exception.get_err() {
                    Ok(error) => Error(format!("{error:?}")),
                    Err(exception) => match exception.get_halt() {
                        Ok(code) => Error(format!("the program halted, with exit code {code}")),
                        Err(_) => Error("the program ended in a way that is not an error".to_string()),
                    },
                }),
            })
        }))
    }
}

/// Every value `filter` yields over `input`, in order, collected. A
/// program that does not parse, does not compile, or fails as it runs
/// is the [`Error`], in jaq's own words; values it yielded before a
/// failure are not returned. For a program run once, over one value; a
/// program run over many is a [`Program`].
pub fn run(filter: &str, input: Value) -> Result<Vec<Value>, Error> {
    Program::new(filter)?.run(input).collect()
}

/// Why a jq program could not be run: it did not parse, did not
/// compile, or failed as it ran, in jaq's own words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the jq program: {}", self.0)
    }
}

impl std::error::Error for Error {}
