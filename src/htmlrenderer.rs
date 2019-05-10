use failure::Error;
use serde::Serialize;

pub struct HTMLRenderer {
    #[cfg(not(debug_assertions))]
    hbars: handlebars::Handlebars,
}

impl HTMLRenderer {

    pub fn new() -> HTMLRenderer {

        #[cfg(not(debug_assertions))]
        {
            let mut hbars = handlebars::Handlebars::new();

            assert!(hbars.register_templates_directory(".html", "templates").is_ok());

            HTMLRenderer {
                hbars
            }
        }

        #[cfg(debug_assertions)]
        HTMLRenderer {}
    }

    pub fn render_page<T>(&self, filename: &str, data: &T) -> Result<String, Error> 
        where T: Serialize
    {

        #[cfg(not(debug_assertions))]
        let result = {
            self.hbars.render(filename, &data)?

        };

        #[cfg(debug_assertions)]
        let result = {
            let mut hbars = handlebars::Handlebars::new();

            assert!(hbars.register_templates_directory(".html", "templates").is_ok());

            hbars.render(filename, &data)?
        };

        Ok(result)
    }
}

