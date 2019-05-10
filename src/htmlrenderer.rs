use failure::Error;
use std::collections::BTreeMap;

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

    pub fn render_page(&self, filename: &str) -> Result<String, Error> {

        let mut data = BTreeMap::new();
        data.insert("world".to_string(), "世界!".to_string());


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

