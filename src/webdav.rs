use std::fmt::Display;
use std::io::{Write, self};
use std::time::SystemTime;

pub fn respond_propfind_dir(xml: &mut impl Write, settings: &crate::Settings, root: &str, dir: &crate::fs::dir::Snapshot, depth: Option<u8>) -> io::Result<()> {
    debug_assert!(root.starts_with("/") && root.ends_with("/"));
    let depth = depth.unwrap_or(!0);

    writeln!(xml, r#"<?xml version="1.0" encoding="utf-8" ?>"#)?;
    writeln!(xml, r#"<multistatus xmlns="DAV:"> "#)?;
    response_dir(xml, settings, root, dir, depth)?;
    writeln!(xml, r#"</multistatus>"#)?;
    return Ok(());

    fn response_dir(xml: &mut impl Write, settings: &crate::Settings, root: &str, dir: &crate::fs::dir::Snapshot, depth: u8) -> io::Result<()> {
        writeln!(xml, r#"  <response>"#)?;
        writeln!(xml, r#"    <href>{root}</href>"#)?;
        writeln!(xml, r#"    <propstat>"#)?;
        writeln!(xml, r#"      <prop>"#)?;
        //writeln!(xml, r#"        <creationdate>1997-12-01T18:27:21-08:00</creationdate>"#)?;
        writeln!(xml, r#"        <displayname>{}</displayname>"#, dir.path().file_name().map_or("Untitled".into(), |os| os.to_string_lossy()))?;
        //writeln!(xml, r#"        <getcontentlength>9001</getcontentlength>"#)?;
        //writeln!(xml, r#"        <getcontenttype>text/html</getcontenttype>"#)?;
        //writeln!(xml, r#"        <getetag>"etag"</getetag>"#)?;
        //writeln!(xml, r#"        <getlastmodified>Mon, 12 Jan 1998 09:25:56 GMT<getlastmodified>"#)?;
        writeln!(xml, r#"        <resourcetype><collection/></resourcetype>"#)?;
        //writeln!(xml, r#"        <supportedlock>"#)?;
        //writeln!(xml, r#"          <lockentry><lockscope><exclusive/></lockscope><locktype><write/></locktype></lockentry>"#)?;
        //writeln!(xml, r#"          <lockentry><lockscope><shared/></lockscope><locktype><write/></locktype></lockentry>"#)?;
        //writeln!(xml, r#"        </supportedlock>"#)?;
        writeln!(xml, r#"      </prop>"#)?;
        writeln!(xml, r#"      <status>HTTP/1.0 200 OK</status>"#)?;
        writeln!(xml, r#"    </propstat>"#)?;
        writeln!(xml, r#"  </response>"#)?;

        if let Some(depth) = depth.checked_sub(1) {
            for e in dir.entries() {
                let name = e.name_lossy();
                if e.is_dir() {
                    let subdir = settings.cache.read_dir(e.path()).ok_or(io::ErrorKind::Other)?;
                    response_dir(xml, settings, &format!("{root}{name}/"), &subdir, depth)?;
                } else if e.is_file() {
                    writeln!(xml, r#"  <response>"#)?;
                    writeln!(xml, r#"    <href>{root}{name}</href>"#)?;
                    writeln!(xml, r#"    <propstat>"#)?;
                    writeln!(xml, r#"      <prop>"#)?;
                    writeln!(xml, r#"        <displayname>{name}</displayname>"#)?;
                    writeln!(xml, r#"        <resourcetype/>"#)?;
                    if let Ok(meta) = e.path().metadata() {
                        writeln!(xml, r#"        <getcontentlength>{}</getcontentlength>"#, meta.len())?;
                        if let Ok(Ok(created)) = meta.created().map(|t| DateTimeUTC::try_from(t)) {
                            writeln!(xml, r#"        <creationdate>{}</creationdate>"#, created.creationdate_style())?;
                        }
                        if let Ok(Ok(modified)) = meta.modified().map(|t| DateTimeUTC::try_from(t)) {
                            writeln!(xml, r#"        <getlastmodified>{}</getlastmodified>"#, modified.getlastmodified_style())?;
                        }
                    }
                    //writeln!(xml, r#"        <getcontenttype>text/html</getcontenttype>"#)?;
                    //writeln!(xml, r#"        <getetag>"etag"</getetag>"#)?;
                    //writeln!(xml, r#"        <supportedlock>"#)?;
                    //writeln!(xml, r#"          <lockentry><lockscope><exclusive/></lockscope><locktype><write/></locktype></lockentry>"#)?;
                    //writeln!(xml, r#"          <lockentry><lockscope><shared/></lockscope><locktype><write/></locktype></lockentry>"#)?;
                    //writeln!(xml, r#"        </supportedlock>"#)?;
                    writeln!(xml, r#"      </prop>"#)?;
                    writeln!(xml, r#"      <status>HTTP/1.0 200 OK</status>"#)?;
                    writeln!(xml, r#"    </propstat>"#)?;
                    writeln!(xml, r#"  </response>"#)?;
                } else {
                    // ...?
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)] struct DateTimeUTC {
    pub year:       u32,// 1+ (e.g. 2023)
    pub month_no:   u8, // 1 ..= 12
    pub day_no:     u8, // 1 ..= 31
    pub hour:       u8, // 0 ..= 24
    pub minute:     u8, // 0 ..= 59
    pub second:     u8, // 0 ..= 60
    dow:            u8, // 0 ..= 6 (0 = Thursday)
}

impl DateTimeUTC {
    /// Styled in [`creationdate`](http://www.webdav.org/specs/rfc4918.html#PROPERTY_creationdate) /
    /// [\[RFC3339\]](https://www.rfc-editor.org/rfc/rfc3339) style, e.g.:
    ///
    /// ```text
    /// 1997-12-01T17:42:21-08:00
    /// ```
    pub fn creationdate_style(&self) -> impl Display {
        let Self { year, month_no, day_no, hour, minute, second, dow: _ } = *self;
        format!("{year}-{month_no:02}-{day_no:02}T{hour:02}:{minute:02}:{second:02}-00:00")
    }

    /// Styled in [`getlastmodified`](http://www.webdav.org/specs/rfc4918.html#PROPERTY_getlastmodified) /
    /// [RFC2616 § 14.29 Last-Modified](https://www.rfc-editor.org/rfc/rfc2616#section-14.29) style, e.g.:
    ///
    /// ```text
    /// Mon, 12 Jan 1998 09:25:56 GMT
    /// ```
    pub fn getlastmodified_style(&self) -> impl Display {
        let Self { year, month_no, day_no, hour, minute, second, dow } = *self;
        let month = ["", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].get(usize::from(month_no)).copied().unwrap_or("");
        debug_assert!(month != "");
        let dow = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"][usize::from(dow)];
        format!("{dow}, {day_no} {month} {year} {hour:02}:{minute:02}:{second:02} GMT")
    }

    pub fn from_seconds_since_epoch(seconds_since_epoch: u64) -> Self {
        fn rem_u8(n: u64, d: u8) -> u8 { (n % u64::from(d)) as u8 }

        // epoch is 1970-01-01 00:00:00 UTC
        let second                  = rem_u8(seconds_since_epoch , 60);
        let minute                  = rem_u8(seconds_since_epoch / 60 , 60);
        let hour                    = rem_u8(seconds_since_epoch / 60 / 60 , 24);
        let days_since_epoch        = seconds_since_epoch / 60 / 60 / 24;
        let dow                     = rem_u8(days_since_epoch, 7); // dow = 0 = Thursday

        let mut days = days_since_epoch % (400*365+97);
        let mut t = Self { year: (1970 + 400 * (days_since_epoch / (400*365+97))) as _, month_no: 1, day_no: 1, hour, minute, second, dow };

        while days >= days_in_year(t.year) { // TODO: optimize more
            days -= days_in_year(t.year);
            t.year += 1;
        }
        let is_leap_year = is_leap_year(t.year);

        for month_days in [
            31, 28 + is_leap_year as u32, 31, 30,   // Jan, Feb, Mar, Apr
            31, 30, 31, 31,                         // May, Jun, Jul, Aug
            30, 31, 30, 31,                         // Sep, Oct, Nov, Dec
        ].iter().copied() {
            if let Some(more) = days.checked_sub(month_days.into()) {
                days = more;
                t.month_no += 1;
            } else {
                t.day_no = 1 + days as u8;
                return t;
            }
        }

        panic!("bug: days in year out of bounds");
    }
}

impl TryFrom<SystemTime> for DateTimeUTC {
    type Error = ();
    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        Ok(Self::from_seconds_since_epoch(value.duration_since(SystemTime::UNIX_EPOCH).map_err(|_| ())?.as_secs()))
    }
}

const fn is_leap_year(year: u32) -> bool { year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) }
const fn days_in_year(year: u32) -> u64 { if is_leap_year(year) { 366 } else { 365 } }

#[test] fn check_format() {
    let epoch = DateTimeUTC::from_seconds_since_epoch(0);
    assert_eq!("1970-01-01T00:00:00-00:00",     epoch.creationdate_style().to_string());
    assert_eq!("Thu, 1 Jan 1970 00:00:00 GMT",  epoch.getlastmodified_style().to_string());
}

#[test] fn check_leap_year_handling() {
    let mut seconds_since_epoch = 0;
    let mut year = 1970;

    while year < 3000 {
        let days_this_year = if is_leap_year(year) { 366 } else { 365 };

        let dt = DateTimeUTC::from_seconds_since_epoch(seconds_since_epoch);
        assert_eq!(year, dt.year);
        assert_eq!(1, dt.month_no);
        assert_eq!(1, dt.day_no);

        for _day in 0 .. days_this_year {
            let dt = DateTimeUTC::from_seconds_since_epoch(seconds_since_epoch);
            assert_eq!(year, dt.year);
            // month_no, day_no
            assert_eq!(0, dt.hour);
            assert_eq!(0, dt.minute);
            assert_eq!(0, dt.second);

            seconds_since_epoch += 24 * 60 * 60;
        }

        year += 1;
    }
}
