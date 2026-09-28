#We need to delay the requests a bit to allow the backend to process a new command
HURL_DELAY=5 hurl --test test.hurl --jobs 1 
