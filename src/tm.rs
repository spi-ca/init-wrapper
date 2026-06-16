use core::ops::Sub;
use core::time::Duration;

use libc::timespec;

pub(crate) struct Timespec {
    pub(crate) ts: timespec,
}

#[cfg(test)]
pub(crate) fn new_timespec() -> Timespec {
    Timespec {
        ts: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
    }
}

impl Sub<Timespec> for Timespec {
    type Output = Duration;

    fn sub(self, other: Timespec) -> Duration {
        let sec = self.ts.tv_sec - other.ts.tv_sec;
        let nsec = self.ts.tv_nsec - other.ts.tv_nsec;
        if nsec < 0 {
            Duration::from_secs(sec as u64) - Duration::from_nanos(nsec.unsigned_abs())
        } else {
            Duration::from_secs(sec as u64) + Duration::from_nanos(nsec.unsigned_abs())
        }
    }
}

impl From<timespec> for Timespec {
    fn from(item: timespec) -> Self {
        Timespec { ts: item }
    }
}

impl From<Timespec> for timespec {
    fn from(value: Timespec) -> timespec {
        value.ts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(sec: i64, nsec: i64) -> Timespec {
        Timespec {
            ts: timespec {
                tv_sec: sec,
                tv_nsec: nsec,
            },
        }
    }

    #[test]
    fn subtracts_timespecs_without_nanosecond_borrow() {
        assert_eq!(
            ts(10, 800) - ts(7, 300),
            Duration::from_secs(3) + Duration::from_nanos(500)
        );
    }

    #[test]
    fn subtracts_timespecs_with_nanosecond_borrow() {
        assert_eq!(
            ts(10, 100) - ts(7, 900),
            Duration::from_secs(2) + Duration::from_nanos(999_999_200)
        );
    }

    #[test]
    fn converts_to_and_from_libc_timespec() {
        let raw = timespec {
            tv_sec: 42,
            tv_nsec: 123,
        };
        let wrapped = Timespec::from(raw);
        let roundtrip: timespec = wrapped.into();
        assert_eq!(roundtrip.tv_sec, 42);
        assert_eq!(roundtrip.tv_nsec, 123);
    }
}
