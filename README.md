# Recense

Recense is a bookmarking service. It can help you store links you don't want to lose, or information you want to remember.

It has been designed to be as simple as possible, so right now it doesn't require a database or any other software installed on the machine. It will store the bookmarks and user data as json files, and use those files for the whole operation.

To run it, you'll need to have a recent version of Rust installed, and then run this command:

    cargo run --release
    
That should start the Recense server. After that, going to http://localhost:8081 should show you your new Recense instance
