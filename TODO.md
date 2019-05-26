### High priority
- Adding a tag containing a dash results in the dash disappearing
- Make tag sizes be proportional to count, but also capped
- Add placeholder images while downloading thumbnail or for broken links
- Fix layout when using long tags
- Deduplicate tags
- Show signup/login errors to user
- Clean design of login page
- Use an html parser (html5ever?) to convert all local references in stored html code into global references

### Medium priority
- Clean up edit_pin_data, it's mostly duplicated code from add_pin
- Add blog section to talk about the project itself?
- Make search query persist when adding/deleting bookmarks 
- Add way to edit user info/password
- Add way to delete accounts
- Add bookmark pagination
- Add server-side logging
	- Add performance tracking to see when we start running into issues with that
- Setup automated backup system
- Implement downloading your bookmarks in a zip file

### Low priority
- Keep track of bookmark creation and modification dates independently?
- Add tag suggestions when entering tag field
- Add other sorting mechanisms for bookmarks
- Add tag sorting options
- Add mobile support to the CSS layout
- Implement getting a website's title to automate link naming?
- Warn when adding the same URL twice?
- Handle failure to download link


## Bulk editing
Being able to perform operations on many bookmarks sounds like it's gonna be important in the future. Should
keep that in mind.

