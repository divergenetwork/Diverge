//! Which secrets a program takes, and how.

use indexmap::IndexMap;

use super::Mode;

/// The program's answer about its secrets: each name it takes, as
/// its image states it, to the [`Mode`] it takes it in, in the
/// order the program names them. A name the template does not
/// declare is the registration's error; a name the image states
/// and this omits is a secret the program does not want, and is never
/// fed. Empty, the program takes none: nothing is ever withheld from
/// it for want of a feed, and nothing is collected from it.
pub type Declaration = IndexMap<String, Mode>;
