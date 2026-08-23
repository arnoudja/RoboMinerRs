//! Business outcome of a DB mutation (success vs domain rejection).
//!
//! Infrastructure failures stay on the outer `Result<_, sqlx::Error>` (or
//! `DomainError` in domain wrappers). Variants are named `Success` / `Rejected`
//! so they do not collide with the prelude `Result::{Ok, Err}` patterns.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbOutcome<T, R> {
    Success(T),
    Rejected(R),
}

impl<T, R> DbOutcome<T, R> {
    #[inline]
    pub fn success(value: T) -> Self {
        Self::Success(value)
    }

    #[inline]
    pub fn rejected(rejection: R) -> Self {
        Self::Rejected(rejection)
    }

    #[inline]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Rejected(_))
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> DbOutcome<U, R> {
        match self {
            Self::Success(v) => DbOutcome::Success(f(v)),
            Self::Rejected(r) => DbOutcome::Rejected(r),
        }
    }

    pub fn map_err<S, F: FnOnce(R) -> S>(self, f: F) -> DbOutcome<T, S> {
        match self {
            Self::Success(v) => DbOutcome::Success(v),
            Self::Rejected(r) => DbOutcome::Rejected(f(r)),
        }
    }

    pub fn into_result(self) -> Result<T, R> {
        match self {
            Self::Success(v) => Ok(v),
            Self::Rejected(r) => Err(r),
        }
    }
}

impl<T, R> From<Result<T, R>> for DbOutcome<T, R> {
    fn from(value: Result<T, R>) -> Self {
        match value {
            Ok(v) => Self::Success(v),
            Err(r) => Self::Rejected(r),
        }
    }
}

impl<T, R> From<DbOutcome<T, R>> for Result<T, R> {
    fn from(value: DbOutcome<T, R>) -> Self {
        value.into_result()
    }
}

/// Replace `Ok(Ok(value))` at mutation API boundaries.
#[inline]
pub fn db_ok<T, R, E>(value: T) -> Result<DbOutcome<T, R>, E> {
    Ok(DbOutcome::Success(value))
}

/// Replace `Ok(Err(rejection))` at mutation API boundaries.
#[inline]
pub fn db_reject<T, R, E>(rejection: R) -> Result<DbOutcome<T, R>, E> {
    Ok(DbOutcome::Rejected(rejection))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_round_trips_with_result() {
        let ok: DbOutcome<i32, &str> = DbOutcome::from(Ok(7));
        assert!(ok.is_ok());
        assert_eq!(ok.into_result(), Ok(7));

        let err: DbOutcome<i32, &str> = DbOutcome::from(Err("nope"));
        assert!(err.is_err());
        assert_eq!(Result::<i32, _>::from(err), Err("nope"));
    }

    #[test]
    fn db_ok_and_reject_helpers() {
        let ok: Result<DbOutcome<u8, &str>, ()> = db_ok(1);
        assert!(matches!(ok, Ok(DbOutcome::Success(1))));
        let rejected: Result<DbOutcome<u8, &str>, ()> = db_reject("x");
        assert!(matches!(rejected, Ok(DbOutcome::Rejected("x"))));
    }
}
