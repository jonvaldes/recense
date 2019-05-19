### High priority
- Find better design for "add pin" and header sections
- Rename view pin to edit
- Fix "edit pin" layout
- When you have a search term it shouldn't show the tutorial, but a message saying no results found
- Implement pin data editting
- Implement deleting/trash can
- Show signup/login errors to user
- Clean design of login page
- Move all styling to the css file, so every page is consistent

### Medium priority
- Add blog section to talk about the project itself?
- Make search query persist when adding pins
- Add way to edit user info
- Add way to delete accounts
- Add pin pagination
- Add server-side analytics/logging
	- Add performance tracking to see when we start running into issues with that
- Setup automated backup system
- Implement downloading your pins in a zip file

### Low priority
- Make the description field support Markdown ??
- Add other sorting mechanisms for pins
- Add tag sorting options
- Add different CSS for "dark mode"
	- Move all color settings to a different CSS file, so we only have to switch that one to get different colors
	(use CSS variables for colors)
- Add mobile support to the CSS
- Finish screenshot taking code. Store PNG image of website, and DOM dump
- Add "object storage"-based image storing system. Store new images for each pin (don't share them between
  pins, as same URL can have different contents depending on when you take the sshot). Also, store html
  contents the same way
- Rescale page image to smaller size, can't afford storing at full size.
- Implement "view pin" page, showing the saved version of the page
- Implement getting a website's title to automate link naming?
- Add support for markdown in pin description
- Warn when adding the same URL twice?
- Handle failure to download link


## Bulk editing
Being able to perform operations on many pins sounds like it's gonna be important in the future. Should keep
that in mind.

