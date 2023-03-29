use std::io::{Write, self};

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
                    let meta = e.path().metadata()?;
                    let len = meta.len();
                    writeln!(xml, r#"  <response>"#)?;
                    writeln!(xml, r#"    <href>{root}{name}</href>"#)?;
                    writeln!(xml, r#"    <propstat>"#)?;
                    writeln!(xml, r#"      <prop>"#)?;
                    //writeln!(xml, r#"        <creationdate>1997-12-01T18:27:21-08:00</creationdate>"#)?;
                    writeln!(xml, r#"        <displayname>{name}</displayname>"#)?;
                    writeln!(xml, r#"        <getcontentlength>{len}</getcontentlength>"#)?;
                    //writeln!(xml, r#"        <getcontenttype>text/html</getcontenttype>"#)?;
                    //writeln!(xml, r#"        <getetag>"etag"</getetag>"#)?;
                    //writeln!(xml, r#"        <getlastmodified>{modified:?}<getlastmodified>"#)?;
                    writeln!(xml, r#"        <resourcetype/>"#)?;
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
