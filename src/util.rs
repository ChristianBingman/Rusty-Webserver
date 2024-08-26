pub mod html {
    use std::path::Path;

    use crate::http10::result_codes::ResultCode;

    pub fn dir_listing(paths: Vec<String>) -> String {
        format!(
            "<html>\n\
                <head>\n\
                    <title>Directory Listing</title>\n\
                </head>\n\
                <body>\n\
                    <ul>\n\
                        <li><a href='../'>../</a></li>\n\
                        {}\n\
                    </ul>\n\
                </body>\n\
            </html>",
            paths
                .iter()
                .map(|path| format!(
                    "<li><a href='{}'>{}</a></li>",
                    &path[1..],
                    Path::new(&path).file_name().unwrap().to_str().unwrap(),
                ))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }

    pub fn error_page(err: ResultCode) -> String {
        format!(
            "<html>\n\
            <head>\n\
                <title>{}</title>\n\
            </head>\n\
            <body>\n\
                <h1>{}</h1>\n\
            </body>\n\
        </html>",
            Into::<String>::into(err),
            Into::<String>::into(err)
        )
    }

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn test_directory_listing() {
            let listing = dir_listing(vec!["./index.html".to_string(), "./banana.php".to_string()]);
            let html = "<html>\n\
                    <head>\n\
                        <title>Directory Listing</title>\n\
                    </head>\n\
                    <body>\n\
                        <ul>\n\
                            <li><a href='../'>../</a></li>\n\
                            <li><a href='/index.html'>index.html</a></li>\n\
                            <li><a href='/banana.php'>banana.php</a></li>\n\
                        </ul>\n\
                    </body>\n\
                </html>";
            assert_eq!(listing, html);
        }

        #[test]
        fn test_directory_listing_subpath() {
            let listing = dir_listing(vec![
                "./src/index.html".to_string(),
                "./yellow/banana.php".to_string(),
            ]);
            let html = "<html>\n\
                    <head>\n\
                        <title>Directory Listing</title>\n\
                    </head>\n\
                    <body>\n\
                        <ul>\n\
                            <li><a href='../'>../</a></li>\n\
                            <li><a href='/src/index.html'>index.html</a></li>\n\
                            <li><a href='/yellow/banana.php'>banana.php</a></li>\n\
                        </ul>\n\
                    </body>\n\
                </html>";
            assert_eq!(listing, html);
        }
    }
}
