Defines the filename format that is used when writing to disk/sent to some uploaders that require a filename.

To allow for uniqueness (which is obviously quite important for filenames), MagicCap supports many different expressions that can be used in filenames:

- `{date}`: Places the date in localized form.
- `{time}`: Places the time in localized form.
- `{random:emoji}`: Places a single random emoji.
- `{random:emoji:N}`: Places `N` emojis. Replace `N` with the number of emojis you want.
- `{random:A-B}`: Selects a random number that is between `A` and `B`. You should replace `A` with the minimum and `B` with the maximum numbers.
- `{random:alphabet}`: Puts a single lowercase character between a and z into the filename.
- `{random:alphabet:N}`: Puts `N` lowercase characters between a and z into the filename. Replace `N` with the number of characters you want.
- `{random:alphabet:A-B}`: Puts a single lowercase character between `A` and `B` into the filename. Both `A` and `B` should be replaced with a character that is between a and z either upper case or lower case. For example, `a-Z` will be a random character in the range a-z or A-Z.
- `{random:alphabet:A-B:N}`: Puts `N` of the above into your filename. Replace `N` with the number of characters you want.
