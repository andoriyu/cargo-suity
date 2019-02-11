use std::io;
use std::convert::From;

#[derive(Debug, Fail)]
pub enum SuityError {
    #[fail(display="Event stream contains results for multiple runs.")]
    MultipleTestRuns,
    #[fail(display="Couldn't locate binary for {} in {} workflow.", name, workflow)]
    TestBinaryNotFound {
        name: String,
        workflow: String,
    },
    #[fail(display="See cause for more information.")]
    IoError(#[fail(cause)] io::Error),
    #[fail(display="Failed to compile tests for {} workflow.", workflow)]
    FailedToCompile {
        workflow: String
    },

}

impl From<io::Error> for SuityError {
    fn from(error: io::Error) -> SuityError {
        SuityError::IoError(error)
    }
}