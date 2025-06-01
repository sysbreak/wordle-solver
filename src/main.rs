use reqwest::Client;
use std::error::Error;
use std::collections::{HashSet, HashMap};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://gist.githubusercontent.com/dracos/dd0668f281e685bad51479e5acaadb93/raw/6bfa15d263d6d5b63840a8e5b64e04b382fdb079/valid-wordle-words.txt";

    let client = Client::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let words: HashSet<String> = body.lines().map(|s| s.to_string().to_lowercase()).collect();

        println!("Number of words loaded: {}", words.len());

        let guess = "roate"; // Hardcoded first guess
        println!("Guess: {}", guess);

        print!("Enter feedback (Green = @, Yellow = #, Gray = ?): ");
        io::stdout().flush()?;

        let mut feedback_input = String::new();
        io::stdin().read_line(&mut feedback_input)?;
        let feedback_input = feedback_input.trim();

        if feedback_input.len() != guess.len() {
            println!("Error: Feedback length must match guess length ({} characters).", guess.len());
            return Ok(());
        }

        println!("Feedback: {}", feedback_input);

        let mut filtered_words: HashSet<String> = words.clone();

        let guess_chars: Vec<char> = guess.chars().collect();
        let feedback_chars: Vec<char> = feedback_input.chars().collect();

        // Word filtering logic (from your original code)
        filtered_words.retain(|word| {
            let mut is_valid = true;
            let word_chars: Vec<char> = word.chars().collect();

            if word_chars.len() != guess_chars.len() {
                return false;
            }

            // Part 1: Greens
            for i in 0..guess_chars.len() {
                if feedback_chars[i] == '@' {
                    if word_chars[i] != guess_chars[i] {
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }

            // Part 2: Yellows
            for i in 0..guess_chars.len() {
                if feedback_chars[i] == '#' {
                    if word_chars[i] == guess_chars[i] {
                        is_valid = false;
                        break;
                    }
                    if !word_chars.contains(&guess_chars[i]) {
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }

            // Part 3: Grays (positional check - gray means not at this spot)
            for i in 0..guess_chars.len() {
                if feedback_chars[i] == '?' {
                    // If a character is marked gray, it shouldn't be in that specific position.
                    // This check is particularly important if the character *also* appears as green/yellow elsewhere.
                    // However, the more crucial aspect of gray is handled by character count logic.
                    // A simple check: if it's gray here, it cannot be this char at this position.
                    // This is implicitly handled by green checks (if it were green, it wouldn't be gray)
                    // and by character count logic (if a char is gray, its count might be exact).
                    // The original code had:
                    // if word_chars[i] == guess_chars[i] { is_valid = false; break; }
                    // This is correct: if feedback for guess_chars[i] is '?', then word_chars[i] cannot be guess_chars[i].
                     if word_chars[i] == guess_chars[i] && feedback_chars.iter().filter(|&&fc| fc == '@' || fc == '#').enumerate().all(|(pi, _)| guess_chars[pi] != guess_chars[i]) {
                        // More nuanced: if guess_chars[i] is gray, AND that same character isn't green/yellow elsewhere in the guess
                        // then word_chars[i] must not be guess_chars[i].
                        // The original logic was simpler and mostly correct for this stage.
                        // The count logic (Part 4) is more robust for grays.
                        // For now, let's keep the original simple gray check for positional non-match if char is truly gray overall.
                        // This check might be redundant given part 4 but doesn't harm.
                         let char_is_elsewhere_green_or_yellow = (0..guess_chars.len()).any(|j| {
                            i != j && guess_chars[j] == guess_chars[i] && (feedback_chars[j] == '@' || feedback_chars[j] == '#')
                        });
                        if !char_is_elsewhere_green_or_yellow && word_chars.contains(&guess_chars[i]) && feedback_chars[i] == '?' {
                            // If truly a gray letter (not appearing as green/yellow elsewhere)
                            // then it shouldn't be in the word AT ALL.
                            // The original loop for part 3 was:
                            // if feedback_chars[i] == '?' { if word_chars[i] == guess_chars[i] { is_valid = false; break; } }
                            // This only prevents it from being at that spot. More robust gray handling is in part 4.
                            // For now, we'll rely on Part 4 to correctly handle exclusion of gray letters.
                            // The provided Part 3 logic was:
                            // if feedback_chars[i] == '?' { if word_chars[i] == guess_chars[i] { is_valid = false; break; }}
                            // This rule is: if a letter is gray at a position, the solution cannot have that letter at that position. This is correct.
                             if word_chars[i] == guess_chars[i] {
                                 is_valid = false;
                                 break;
                             }
                        }
                    }
                }
            }
            if !is_valid { return false; }


            // Part 4: Character counts
            let mut char_info = HashMap::new();
            for i in 0..guess_chars.len() {
                let gc = guess_chars[i];
                let fb = feedback_chars[i];
                let info = char_info.entry(gc).or_insert((0, 0)); // (green_yellow_count, gray_count)
                match fb {
                    '@' | '#' => info.0 += 1,
                    '?' => info.1 += 1,
                    _ => { is_valid = false; /* return false; */ } // Invalid feedback char
                }
            }
            if !is_valid { return false; }


            for (guessed_char, (green_yellow_in_guess, gray_in_guess)) in char_info.iter() {
                let count_in_word = word_chars.iter().filter(|&&wc| wc == *guessed_char).count();

                if *gray_in_guess > 0 {
                    // If a char instance was marked gray (e.g., guess "BOOOK", first 'O' is gray, second is yellow, third is green)
                    // the word must contain this char *exactly* as many times as it was green/yellow in the guess.
                    if count_in_word != *green_yellow_in_guess {
                        is_valid = false;
                        break;
                    }
                } else {
                    // If a char was *only* green or yellow in the guess (no gray instances of this char),
                    // the word must contain it *at least* as many times as it was green/yellow in the guess.
                    if count_in_word < *green_yellow_in_guess {
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }
            
            // Final validation for feedback characters (redundant if input is sanitized, but safe)
            for &fc in feedback_chars.iter() {
                if !['@', '#', '?'].contains(&fc) {
                    // This should be caught earlier, but as a safeguard within the retain closure
                    // println!("Invalid feedback character '{}' encountered during filtering.", fc); // Avoid println in closure
                    is_valid = false;
                    break;
                }
            }

            is_valid
        });

        println!("Number of filtered words: {}", filtered_words.len());

        // --- Start of new logic for suggesting next guess ---

        if feedback_input == "@@@@@" {
            println!("🎉 Congratulations! You guessed the word!");
            return Ok(());
        }

        if filtered_words.is_empty() {
            println!("🤔 No words match the given feedback. Unable to suggest a next guess.");
            return Ok(());
        }

        if filtered_words.len() == 1 {
            let answ = filtered_words.iter().next().unwrap();
            println!("💡 The only remaining word is: {}", answ);
            println!("Suggested next guess: {}", answ); // Suggesting it as the next guess
            return Ok(());
        }

        // Suggest next guess based on frequency scoring for multiple remaining words
        println!("🧠 Calculating best next guess from {} possibilities...", filtered_words.len());

        // 1. Calculate letter frequencies in filtered_words
        let mut letter_frequencies: HashMap<char, usize> = HashMap::new();
        for word_str in filtered_words.iter() { // word_str is &String
            for ch in word_str.chars() {
                *letter_frequencies.entry(ch).or_insert(0) += 1;
            }
        }

        // 2. Score each word in filtered_words as a potential next guess
        let mut best_guess_candidate: Option<String> = None;
        let mut max_score = 0; // Using 0 as initial min score.

        for potential_guess_word_str in filtered_words.iter() { // potential_guess_word_str is &String
            let mut current_score = 0;
            let mut unique_chars_in_guess = HashSet::new();
            for ch in potential_guess_word_str.chars() {
                unique_chars_in_guess.insert(ch);
            }

            for unique_char in unique_chars_in_guess {
                current_score += *letter_frequencies.get(&unique_char).unwrap_or(&0);
            }

            match best_guess_candidate {
                Some(ref current_best_str_ref) => {
                    if current_score > max_score {
                        max_score = current_score;
                        best_guess_candidate = Some(potential_guess_word_str.clone());
                    } else if current_score == max_score {
                        // Tie-breaking: choose alphabetically smaller for deterministic output
                        if potential_guess_word_str < current_best_str_ref {
                             best_guess_candidate = Some(potential_guess_word_str.clone());
                        }
                    }
                }
                None => { // This is the first word being processed
                    max_score = current_score;
                    best_guess_candidate = Some(potential_guess_word_str.clone());
                }
            }
        }

        if let Some(suggested_guess) = best_guess_candidate {
            println!("🎯 Suggested next guess based on frequency scoring: {}", suggested_guess);
        } else {
            // This case should not be reached if filtered_words was non-empty.
            println!("🤷 Could not determine a best guess from the remaining options.");
        }

    } else {
        println!("Failed to fetch words: {}", response.status());
    }

    Ok(())
}