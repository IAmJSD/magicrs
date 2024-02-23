use once_cell::sync::Lazy;
use rand::{seq::SliceRandom, Rng};

// Defines all of the emojis.
static EMOJIS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    emojis::iter()
        .map(|emoji| emoji.as_str())
        .collect()
});

// Defines the regex for emoji randoms.
static EMOJI_RANDOM_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"\{random:emoji(:\d+)?\}").unwrap()
});

// Defines the regex for number randoms.
static NUMBER_RANDOM_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"\{random:(\d+)-(\d+)\}").unwrap()
});

// Defines the regex for alphabet randoms. This defaults to all lowercase letters.
static ALPHABET_RANDOM_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"\{random:alphabet(:\d+)?\}").unwrap()
});

// Defines the regex for alphabet randoms with a range of characters.
static ALPHABET_RANDOM_RANGE_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"\{random:alphabet:([a-zA-Z])-([a-zA-Z])(:\d+)?\}").unwrap()
});

// Defines a function to get a new filename with the edits applied.
pub fn get_filename(filename_tpl: Option<String>) -> Result<String, String> {
    // Get the filename template.
    let filename_tpl = filename_tpl.unwrap_or("screenshot_{date}_{time}".to_owned());

    // Clone the filename template. This is our base.
    let mut filename = filename_tpl.clone();

    // Get the current time in the user's timezone.
    let now = chrono::Local::now();

    // Replace all instances of {date} with the current date in locale format.
    filename = filename.replace(
        "{date}", &now.format("%x").to_string().replace("/", "-")
    );

    // Replace all instances of {time} with the current time in locale format.
    filename = filename.replace(
        "{time}", &now.format("%X").to_string().replace(":", "-")
    );

    loop {
        // Handle random emojis.
        if let Some(captures) = EMOJI_RANDOM_REGEX.captures(&filename) {
            // Get the count of emojis.
            let count = captures.get(1).map_or(1, |m| m.as_str().parse::<usize>().unwrap());

            // Get a random set of emojis.
            let emojis = EMOJIS.
                choose_multiple(&mut rand::thread_rng(), count).
                map(|x| *x).
                collect::<Vec<_>>();

            // Replace the emoji random.
            filename = filename.replacen(
                captures.get(0).unwrap().as_str(), emojis.join("").as_str(),
                1,
            );

            // Continue the loop.
            continue;
        }

        // Handle random numbers.
        if let Some(captures) = NUMBER_RANDOM_REGEX.captures(&filename) {
            // Get the minimum and maximum values.
            let min = captures.get(1).unwrap().as_str().parse::<i64>().unwrap();
            let max = captures.get(2).unwrap().as_str().parse::<i64>().unwrap();

            // Make sure the minimum is less than the maximum.
            if min > max {
                return Err("Minimum value is greater than maximum value in the filename numeric matcher.".to_string());
            }

            // Get a random number.
            let number = rand::thread_rng().gen_range(min..=max);

            // Replace the number random.
            filename = NUMBER_RANDOM_REGEX.replace(&filename, number.to_string().as_str()).to_string();

            // Continue the loop.
            continue;
        }

        // Handle random alphabet characters.
        if let Some(captures) = ALPHABET_RANDOM_REGEX.captures(&filename) {
            // Get the count of alphabets.
            let count = captures.get(1).map_or(1, |m| m.as_str().parse::<usize>().unwrap());

            // Get a random set of alphabets.
            let alphabets = (0..count).map(|_| {
                rand::thread_rng().gen_range(b'a'..=b'z') as char
            }).collect::<String>();

            // Replace the alphabet random.
            filename = filename.replacen(
                captures.get(0).unwrap().as_str(), alphabets.as_str(),
                1,
            );

            // Continue the loop.
            continue;
        }

        // Handle random alphabet characters with a range.
        if let Some(captures) = ALPHABET_RANDOM_RANGE_REGEX.captures(&filename) {
            // Get the minimum and maximum values.
            let min = captures.get(1).unwrap().as_str().chars().next().unwrap() as u8;
            let max = captures.get(2).unwrap().as_str().chars().next().unwrap() as u8;

            // Make sure the minimum is less than the maximum. If it isn't, swap them.
            let (min, max) = if min > max {
                (max, min)
            } else {
                (min, max)
            };

            // Get the count of alphabets.
            let count = captures.get(3).map_or(1, |m| m.as_str().parse::<usize>().unwrap());

            // Get a random set of alphabets.
            let alphabets = (0..count).map(|_| {
                rand::thread_rng().gen_range(min..=max) as char
            }).collect::<String>();

            // Replace the alphabet random.
            filename = filename.replacen(
                captures.get(0).unwrap().as_str(), alphabets.as_str(),
                1,
            );

            // Continue the loop.
            continue;
        }

        // Break the loop.
        break;
    }

    // Return the filename.
    Ok(filename)
}
