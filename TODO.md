### Next
- Switch from chromium browsers to firefox for screenshots and caching. Chromium is extremely unreliable, and
  hangs while taking screenshots.
  - Use mozrunner to control the firefox process: https://crates.io/crates/mozrunner
  - Implement minimal Marionette support in recense to extract screenshots and the DOM for caching

- Add reloading mechanism for the screenshots, so they load automatically when they're ready

### High priority
- Adding a tag containing a dash results in the dash disappearing
- Show signup/login errors to user
- Clean design of login page

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

### Low priority
- Keep track of bookmark creation and modification dates independently?
- Add tag suggestions when entering tag field
- Add other sorting mechanisms for bookmarks
- Add tag sorting options
- Add mobile support to the CSS layout
- Implement getting a website's title to automate link naming?
- Warn when adding the same URL twice?
- Handle failure to download link

### Long-term
- Implement proper REST API to interact with the data

## Bulk editing
Being able to perform operations on many bookmarks sounds like it's gonna be important in the future. Should
keep that in mind.

