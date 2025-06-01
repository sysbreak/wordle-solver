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
        let words: HashSet<String> = body.lines().map(|s| s.to_string().to_lowercase()).collect(); // made it lowercase 

        println!("Number of words: {}", words.len());

        
        let guess = "roate"; 
        println!("Guess: {}", guess);

        print!("Enter feedback (Green = @, Yellow = #, Gray = ?): ");
        io::stdout().flush()?; // display prompt immediately

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

        filtered_words.retain(|word| {
            let mut is_valid = true;
            let word_chars: Vec<char> = word.chars().collect();

            if word_chars.len() != guess_chars.len() { // check length match
                return false;
            }

            // master plan part 1: check where greens are
            for i in 0..guess_chars.len() {
                if feedback_chars[i] == '@' {
                    if word_chars[i] != guess_chars[i] {
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }

            // master plan part 2: check where yellws are
            for i in 0..guess_chars.len() {
                if feedback_chars[i] == '#' {
                    if word_chars[i] == guess_chars[i] { // yellow means its not here gng
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
            
            // master plan part 3L check where grays are
            for i in 0..guess_chars.len() {
                if feedback_chars[i] == '?' {
                    if word_chars[i] == guess_chars[i] {
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }


            // master plan part 4: check counts of chars
            let mut char_info = HashMap::new();

            for i in 0..guess_chars.len() {
                let gc = guess_chars[i];
                let fb = feedback_chars[i];
                // entry: (count_in_guess_green_yellow, count_in_guess_gray)
                let info = char_info.entry(gc).or_insert((0, 0));
                match fb {
                    '@' | '#' => info.0 += 1,
                    '?' => info.1 += 1,
                    _ => { // invalid feedback character
                        is_valid = false; //  handle error appropriately
                        // no break here
                    }
                }
            }
             if !is_valid { return false; }


            for (guessed_char, (green_yellow_in_guess, gray_in_guess)) in char_info.iter() {
                let count_in_word = word_chars.iter().filter(|&&wc| wc == *guessed_char).count();

                if *gray_in_guess > 0 {
                    // if a char was marked gray in the guess (ex. guess "LEVEL", first L is gray, second L is green)
                    // the word must contain this char *exactly* as many times as it was green/yellow in the guess
                    if count_in_word != *green_yellow_in_guess {
                        is_valid = false;
                        break;
                    }
                } else {
                    // If a char was *only* green or yellow (or not present) in the guess,
                    // the word must contain it *at least* as many times as it was green/yellow in the guess.
                    if count_in_word < *green_yellow_in_guess {
                        is_valid = false;
                        break;
                    }
                }
            }
            if !is_valid { return false; }
            
            //  make sure invalid characters are not present in the feedback
            // this is a bit redundant, but it makes sure we don't have invalid chars in the feedback
            for &fc in feedback_chars.iter() {
                if !['@', '#', '?'].contains(&fc) {

                    println!("Invalid feedback character '{}' encountered during filtering.", fc);
                    is_valid = false; 
                    break;
                }
            }

            is_valid
        });

        println!("Number of filtered words: {}", filtered_words.len());

    } else {
        println!("Failed to fetch words: {}", response.status());
    }

    Ok(())
}