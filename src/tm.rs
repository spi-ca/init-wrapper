use core::ops::Sub;
use core::time::Duration;

use libc::timespec;

pub(crate) struct Timespec {
    pub(crate) ts: timespec,
}

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

impl Into<timespec> for Timespec {
    fn into(self) -> timespec {
        self.ts
    }
}
