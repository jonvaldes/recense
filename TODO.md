### High priority
- Write automated deployment scripts
- Add link to TO-DO when you're logged in
- Implement deleting/trash can
- Make signup log you in
- Show signup/login errors to user
- Implement tag-based filtering
- Implement searching through pins (including tag-filtered searches)
- Clean design of login page
- Make sure font usage complies with license [https://fontlibrary.org/en/font/glacial-indifference]()
- Validate tags as simple lowercase text
- Sort pins by age by default
- Find better design for "add pin" section

### Medium priority
- Add way to edit user info
- Add way to delete accounts
- Add pin pagination
- Add server-side analytics/logging
- Implement "view pin" page, showing the saved version of the page
- Setup automated backup system
- Implement pin data editting

### Low priority
- Add way to bring down recen.se server for maintenance and still show something to users
- Add other sorting mechanisms for pins
- Add different CSS for "dark mode"
- Add mobile support to the CSS
- Make auth cookie survive restarts
- Finish screenshot taking code. Store PNG image of website, and DOM dump
- Add "object storage"-based image storing system. Store new images for each pin (don't share them between
  pins, as same URL can have different contents depending on when you take the sshot). Also, store html
  contents the same way
- Rescale page image to smaller size, can't afford storing at full size.
- Implement downloading your pins in a zip file
- Implement getting a website's title to automate link naming?
- Add support for markdown in pin description
- Warn when adding the same URL twice?
- Handle failure to download link

