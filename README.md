# Password Game Bot

A bot to beat [The Password Game](https://neal.fun/password-game/).

## Running

`cargo r --bin main` will spawn a Chrome window and play the game. Make sure to not touch
the Chrome window as focus on the password box is required for things to work (as we send
key presses directly to the active window, as it's much faster than using the Chrome
DevTools API).

## Known Issues

- We don't have a video URL for all possible YouTube video durations.
- Some YouTube videos are private, deleted, non-embeddable, etc, and some seem to have
  changed lengths.
- About half of the YouTube video URLs contain non-zero digits, chemical element symbols,
  and/or roman numerals, some of which make beating the game impossible (e.g., if the URL
  contains an "M", you're never going to satisfy the roman numeral multiplication rule).
- There are various occassional bugs with input, most notably in the formatting getting out
  of sync between our internal representation and the actual formatting in the game.
- It just hangs sometimes, I think in the DevTools API code. I haven't looked into it much.
