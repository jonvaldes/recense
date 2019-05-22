### High priority
- Deduplicate tags
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
- Implement downloading your bookmarks in a zip file

### Low priority
- Keep track of bookmark creation and modification dates independently?
- Add tag suggestions when entering tag field
- Expand clickable area of links? (and edit/delete buttons?)
- Add other sorting mechanisms for bookmarks
- Add tag sorting options
- Add mobile support to the CSS layout
- Finish screenshot taking code. Store PNG image of website, and DOM dump
- Add "object storage"-based image storing system. Store new images for each bookmark (don't share them
  between bookmarks, as same URL can have different contents depending on when you take the sshot). Also,
  store html contents the same way
- Rescale page image to smaller size, can't afford storing at full size.
- Implement getting a website's title to automate link naming?
- Warn when adding the same URL twice?
- Handle failure to download link


## Bulk editing
Being able to perform operations on many bookmarks sounds like it's gonna be important in the future. Should
keep that in mind.

