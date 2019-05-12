- Show signup/login errors to user
- Validate tags as simple lowercase text
- Setup recen.se server
- Setup letsencrypt for new domain
- Add SSL support
- Setup automated backup system
- Add mobile support to the CSS
- Add different CSS for "dark mode"
- Clean design of login page
- Make signup log you in
- Add way to delete accounts
- Add server-side analytics/logging
- Add pin pagination
- Implement "view pin" page, showing the saved version of the page
- Implement deleting/trash can
- Implement tag-based filtering
- Implement searching through pins (including tag-filtered searches)
- Implement downloading your pins in a zip file
- Implement getting a website's title to automate link naming?
- Handle adding the same URL twice -> Decouple URL from pin ID?

### Taking screenshots: 

Supposedly Firefox can take screenshots in a "headless" mode it has. I've
 been unable to make it work, though. In theory this should be how you do that:

      firefox -no-remote -url https://recen.se/ -screenshot test.jpg

(add "-P <profilename>" to make it use another profile and allow several instances of Firefox running)

See: [https://developer.mozilla.org/en-US/docs/Mozilla/Firefox/Headless_mode]()

Note: That website also shows how to use the Selenium client to store a website's page
source. I could use that to do waybackmachine-like storing of websites
See: [https://www.seleniumhq.org/projects/webdriver/]()

