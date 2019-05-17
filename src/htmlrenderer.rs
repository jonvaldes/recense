use chrono::prelude::DateTime;
use chrono::FixedOffset;
use failure::Error;
use serde::Serialize;

pub struct HTMLRenderer {
    #[cfg(not(debug_assertions))]
    hbars: handlebars::Handlebars,
}

fn format_datetime(v: &str) -> String {
    let dt = match DateTime::<FixedOffset>::parse_from_rfc3339(&v) {
        Err(x) => {
            error!("{}", x);
            return String::from("unknown");
        }
        Ok(x) => x,
    };

    let time_passed = chrono::Utc::now().signed_duration_since(dt);

    let day = time_passed.num_days();
    let year = day / 365;
    let month = day / 30;
    let week = time_passed.num_weeks();
    let hour = time_passed.num_hours();
    let minute = time_passed.num_minutes();

    macro_rules! format_ago {
        ($e: expr) => {
            if $e > 0 {
                return format!(
                    "{} {}{} ago",
                    $e,
                    stringify!($e),
                    if $e > 1 { "s" } else { "" }
                );
            }
        };
    }

    format_ago!(year);
    format_ago!(month);
    format_ago!(week);
    format_ago!(day);
    format_ago!(hour);
    format_ago!(minute);

    String::from("just now")
}

handlebars_helper!(format_time: |s: str| format_datetime(s) );
handlebars_helper!(is_empty_string: |s: str| s == "" );

impl HTMLRenderer {
    pub fn new() -> HTMLRenderer {
        #[cfg(not(debug_assertions))]
        {
            let mut hbars = handlebars::Handlebars::new();

            hbars.register_helper("format_time", Box::new(format_time));
            hbars.register_helper("is_empty_string", Box::new(is_empty_string));

            if let Err(err) = hbars.register_templates_directory(".html", "templates") {
                error!("Error loading HTML templates: {}", err);
                panic!("no idea what to do now");
            }

            HTMLRenderer { hbars }
        }

        #[cfg(debug_assertions)]
        HTMLRenderer {}
    }

    pub fn render_page<T>(&self, filename: &str, data: &T) -> Result<String, Error>
    where
        T: Serialize,
    {
        #[cfg(not(debug_assertions))]
        let result = { self.hbars.render(filename, &data)? };

        #[cfg(debug_assertions)]
        let result = {
            let mut hbars = handlebars::Handlebars::new();

            hbars.register_helper("format_time", Box::new(format_time));
            hbars.register_helper("is_empty_string", Box::new(is_empty_string));

            if let Err(err) = hbars.register_templates_directory(".html", "templates") {
                error!("Error loading HTML templates: {}", err);
                panic!("no idea what to do now");
            }

            hbars.render(filename, &data)?
        };

        Ok(result)
    }
}

pub fn render_markdown_file(filename: &str) -> Result<String, Error> {
    use pulldown_cmark::{html, Options, Parser};

    let markdown_input = std::fs::read_to_string(filename)?;

    let parser = Parser::new_ext(&markdown_input, Options::all());

    // Write to String buffer.
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(html_output)
}
