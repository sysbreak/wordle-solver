use reqwest::Client;
use std::error::Error;
use std::collections::{HashSet, HashMap};
use std::io::{self, Write};

fn get_feedback_pattern(guess: &str, target: &str) -> String {
    let guess_chars: Vec<char> = guess.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();
    let mut result = vec!['?'; guess_chars.len()];
    let mut target_char_counts = HashMap::new();
    
    // Count characters in target
    for ch in target_chars.iter() {
        *target_char_counts.entry(*ch).or_insert(0) += 1;
    }
    
    // First pass: mark greens and reduce counts
    for i in 0..guess_chars.len() {
        if guess_chars[i] == target_chars[i] {
            result[i] = '@';
            *target_char_counts.get_mut(&guess_chars[i]).unwrap() -= 1;
        }
    }
    
    // Second pass: mark yellows
    for i in 0..guess_chars.len() {
        if result[i] == '?' {
            if let Some(count) = target_char_counts.get_mut(&guess_chars[i]) {
                if *count > 0 {
                    result[i] = '#';
                    *count -= 1;
                }
            }
        }
    }
    
    result.into_iter().collect()
}

fn calculate_entropy(guess: &str, possible_words: &HashSet<String>) -> f64 {
    let mut pattern_counts: HashMap<String, usize> = HashMap::new();
    
    // Count how many words produce each feedback pattern
    for target_word in possible_words.iter() {
        let pattern = get_feedback_pattern(guess, target_word);
        *pattern_counts.entry(pattern).or_insert(0) += 1;
    }
    
    // Calculate entropy: -Σ(p * log2(p))
    let total_words = possible_words.len() as f64;
    let mut entropy = 0.0;
    
    for count in pattern_counts.values() {
        if *count > 0 {
            let probability = *count as f64 / total_words;
            entropy -= probability * probability.log2();
        }
    }
    
    entropy
}

async fn play_wordle_game(initial_words: &HashSet<String>) -> Result<(), Box<dyn Error>> {
    let mut filtered_words = initial_words.clone();
    let mut guess_count = 0;
    const MAX_GUESSES: usize = 6; // Standard Wordle guess limit
    let mut rejected_words: HashSet<String> = HashSet::new();

    loop {
        guess_count += 1;
        println!("\n--- Guess #{} ---", guess_count);

        let current_guess: String;

        // 1. Determine the current guess
        if guess_count == 1 {
            current_guess = "roate".to_string(); // A common starting word
            println!("Starting with guess: {}", current_guess);
        } else {
            if filtered_words.is_empty() {
                println!("No possible words remain. Cannot make a guess.");
                break; // End game
            }
            if filtered_words.len() == 1 {
                current_guess = filtered_words.iter().next().unwrap().clone();
                println!("Only one word remains: {}", current_guess);
            } else {
                // --- Entropy Scoring Logic to determine next guess ---
                println!("Calculating best next guess from {} possibilities using entropy scoring...", filtered_words.len());
                
                let mut best_guess_candidate: Option<String> = None;
                let mut max_entropy = 0.0;

                for potential_guess_word_str in filtered_words.iter() {
                    // Skip rejected words
                    if rejected_words.contains(potential_guess_word_str) {
                        continue;
                    }

                    let entropy = calculate_entropy(potential_guess_word_str, &filtered_words);

                    match best_guess_candidate {
                        Some(ref current_best_str_ref) => {
                            if entropy > max_entropy {
                                max_entropy = entropy;
                                best_guess_candidate = Some(potential_guess_word_str.clone());
                            } else if (entropy - max_entropy).abs() < 0.0001 {
                                // If entropies are essentially equal, pick lexicographically smaller
                                if potential_guess_word_str < current_best_str_ref {
                                    best_guess_candidate = Some(potential_guess_word_str.clone());
                                }
                            }
                        }
                        None => {
                            max_entropy = entropy;
                            best_guess_candidate = Some(potential_guess_word_str.clone());
                        }
                    }
                }

                current_guess = match best_guess_candidate {
                    Some(guess) => guess,
                    None => { // Should not happen if filtered_words is not empty here
                        println!("Error: Could not determine a guess. All remaining words have been rejected.");
                        break;
                    }
                };
                println!("Suggested next guess: {} (entropy: {:.3})", current_guess, max_entropy);
                // --- End of Entropy Scoring Logic ---
            }
        }

        // 2. Get feedback from the user
        print!("Enter feedback for '{}' (Green = @, Yellow = #, Gray = ?, or type 'reject' to get a new guess): ", current_guess);
        io::stdout().flush()?;
        let mut feedback_input = String::new();
        io::stdin().read_line(&mut feedback_input)?;
        let feedback_input_trimmed = feedback_input.trim().to_lowercase();

        // Handle rejection
        if feedback_input_trimmed == "reject" {
            println!("Rejecting guess '{}'", current_guess);
            rejected_words.insert(current_guess.clone());
            filtered_words.remove(&current_guess);
            guess_count -= 1; // Don't increment guess count for rejected guesses
            continue;
        }

        // 3. Validate feedback
        if feedback_input_trimmed.len() != current_guess.len() {
            println!("Error: Feedback length ({} chars) must match guess length ({} chars for '{}'). Please try again.",
                     feedback_input_trimmed.len(), current_guess.len(), current_guess);
            guess_count -= 1; // Decrement to retry the same guess number
            continue; // Restart this iteration of the loop to re-enter feedback
        }

        let mut valid_feedback_chars = true;
        for fc in feedback_input_trimmed.chars() {
            if !['@', '#', '?'].contains(&fc) {
                valid_feedback_chars = false;
                break;
            }
        }
        if !valid_feedback_chars {
            println!("Error: Feedback contains invalid characters. Use only '@' (Green), '#' (Yellow), '?' (Gray), or 'reject'. Please try again for guess '{}'.", current_guess);
            guess_count -=1; // Decrement to retry this guess
            continue; // Restart this iteration
        }

        // 4. Check for win condition
        if feedback_input_trimmed == "@@@@@" {
            println!("🎉 Congratulations! You found the word '{}' in {} guesses!", current_guess, guess_count);
            break; // End game
        }

        // 5. Filter words based on current_guess and feedback_input
        let guess_chars_vec: Vec<char> = current_guess.chars().collect();
        let feedback_chars_vec: Vec<char> = feedback_input_trimmed.chars().collect();

        filtered_words.retain(|word| {
            let mut is_valid = true;
            let word_chars: Vec<char> = word.chars().collect();

            if word_chars.len() != guess_chars_vec.len() { // Should not happen if initial list is clean
                return false;
            }

            // Part 1: Greens
            for i in 0..guess_chars_vec.len() {
                if feedback_chars_vec[i] == '@' {
                    if word_chars[i] != guess_chars_vec[i] {
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }

            // Part 2: Yellows
            for i in 0..guess_chars_vec.len() {
                if feedback_chars_vec[i] == '#' {
                    if word_chars[i] == guess_chars_vec[i] { // Yellow means it's NOT at this position
                        is_valid = false;
                        break;
                    }
                    if !word_chars.contains(&guess_chars_vec[i]) { // Yellow means it IS in the word
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }

            // Part 3: Grays (positional non-match)
            // If feedback for guess_chars_vec[i] is '?', then word_chars[i] cannot be guess_chars_vec[i].
            for i in 0..guess_chars_vec.len() {
                if feedback_chars_vec[i] == '?' {
                    if word_chars[i] == guess_chars_vec[i] {
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }

            // Part 4: Character counts based on feedback
            let mut char_info = HashMap::new(); // char -> (green_yellow_count, gray_count)
            for i in 0..guess_chars_vec.len() {
                let gc = guess_chars_vec[i];
                let fb = feedback_chars_vec[i];
                let info = char_info.entry(gc).or_insert((0, 0));
                match fb {
                    '@' | '#' => info.0 += 1,
                    '?' => info.1 += 1,
                    _ => {} // Already validated, so this case shouldn't be hit.
                           // If it were, is_valid = false; would be appropriate.
                }
            }

            for (guessed_char, (green_yellow_in_guess, gray_in_guess)) in char_info.iter() {
                let count_in_word = word_chars.iter().filter(|&&wc| wc == *guessed_char).count();

                if *gray_in_guess > 0 {
                    // If a char was marked gray AT LEAST ONCE in the guess
                    // (e.g., guess "SASSY", S at pos 1 is gray, S at pos 3 is green, S at pos 4 is yellow)
                    // then the target word must contain this char *exactly* as many times as it was green/yellow in the guess.
                    if count_in_word != *green_yellow_in_guess {
                        is_valid = false;
                        break;
                    }
                } else {
                    // If a char was *only* green or yellow (never gray for this char type in the guess)
                    // the target word must contain it *at least* as many times as it was green/yellow in the guess.
                    if count_in_word < *green_yellow_in_guess {
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }

            is_valid
        });
        // --- End of filtering logic ---

        println!("Number of possible words remaining: {}", filtered_words.len());

        // 6. Check game state after filtering
        if filtered_words.is_empty() && feedback_input_trimmed != "@@@@@" {
            println!("🤔 No words match the latest feedback. The target word might not be in the list or an error in feedback occurred.");
            break; // End game
        }
       
        if guess_count >= MAX_GUESSES && feedback_input_trimmed != "@@@@@" {
            println!("😥 You've reached the maximum of {} guesses.", MAX_GUESSES);
            if !filtered_words.is_empty() {
                println!("Possible word(s) could have been: {:?}", filtered_words.iter().take(5).collect::<Vec<_>>());
            } else {
                println!("No words were left as possibilities.");
            }
            break; // End game
        }
    } // End of game loop
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://gist.githubusercontent.com/dracos/dd0668f281e685bad51479e5acaadb93/raw/6bfa15d263d6d5b63840a8e5b64e04b382fdb079/valid-wordle-words.txt";

    let client = Client::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let initial_words: HashSet<String> = body.lines().map(|s| s.to_string().to_lowercase()).collect();

        if initial_words.is_empty() {
            println!("Word list is empty! Cannot start the game.");
            return Ok(());
        }
        println!("Loaded {} valid Wordle words.", initial_words.len());

        loop {
            // Play a game
            play_wordle_game(&initial_words).await?;

            // Ask if user wants to play again
            print!("\nWould you like to play again? (yes/no): ");
            io::stdout().flush()?;
            let mut play_again_input = String::new();
            io::stdin().read_line(&mut play_again_input)?;
            let play_again_trimmed = play_again_input.trim().to_lowercase();

            if play_again_trimmed == "yes" || play_again_trimmed == "y" {
                println!("\n🎮 Starting a new game!\n");
                continue;
            } else {
                println!("Thanks for playing! Goodbye! 👋");
                break;
            }
        }
    } else {
        println!("Failed to fetch words: {}", response.status());
    }
    Ok(())
}