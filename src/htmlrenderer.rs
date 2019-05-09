use failure::Error;
use std::collections::BTreeMap;

pub struct HTMLRenderer {
    hbars: handlebars::Handlebars,
}

impl HTMLRenderer {

    pub fn new() -> HTMLRenderer {

        let mut hbars = handlebars::Handlebars::new();

        assert!(hbars.register_templates_directory(".html", "templates").is_ok());

        HTMLRenderer {
            hbars
        }
    }

    pub fn render_page(&self, filename: &str) -> Result<String, Error> {

        let mut data = BTreeMap::new();
        data.insert("world".to_string(), "世界!".to_string());

        let result = self.hbars.render(filename, &data)?;
        Ok(result)
    }



}
